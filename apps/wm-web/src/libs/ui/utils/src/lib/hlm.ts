import { isPlatformBrowser } from '@angular/common';
import {
	DestroyRef,
	effect,
	ElementRef,
	HostAttributeToken,
	inject,
	Injector,
	PLATFORM_ID,
	runInInjectionContext,
} from '@angular/core';
import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function hlm(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

const elementClassManagers = new WeakMap<HTMLElement, ElementClassManager>();
let globalObserver: MutationObserver | null = null;
const observedElements = new Set<HTMLElement>();

interface ElementClassManager {
	element: HTMLElement;
	sources: Map<number, { classes: Set<string>; order: number }>;
	baseClasses: Set<string>;
	isUpdating: boolean;
	nextOrder: number;
	hasInitialized: boolean;
	restoreRafId: number | null;
	transitionsSuppressed: boolean;
	previousTransition: string;
	previousTransitionPriority: string;
}

let sourceCounter = 0;

export function classes(computed: () => ClassValue[] | string, options: ClassesOptions = {}) {
	runInInjectionContext(options.injector ?? inject(Injector), () => {
		const elementRef = options.elementRef ?? inject(ElementRef);
		const platformId = inject(PLATFORM_ID);
		const destroyRef = inject(DestroyRef);
		const baseClasses = inject(new HostAttributeToken('class'), { optional: true });

		const element = elementRef.nativeElement;
		const sourceId = sourceCounter++;
		let manager = elementClassManagers.get(element);

		if (!manager) {
			const initialBaseClasses = new Set<string>();
			if (baseClasses) {
				toClassList(baseClasses).forEach((cls) => initialBaseClasses.add(cls));
			}
			manager = {
				element,
				sources: new Map(),
				baseClasses: initialBaseClasses,
				isUpdating: false,
				nextOrder: 0,
				hasInitialized: false,
				restoreRafId: null,
				transitionsSuppressed: false,
				previousTransition: '',
				previousTransitionPriority: '',
			};
			elementClassManagers.set(element, manager);
			setupGlobalObserver(platformId);
			observedElements.add(element);
			if (isPlatformBrowser(platformId)) {
				manager.previousTransition = element.style.getPropertyValue('transition');
				manager.previousTransitionPriority = element.style.getPropertyPriority('transition');
				element.style.setProperty('transition', 'none', 'important');
				manager.transitionsSuppressed = true;
			}
		}

		const sourceOrder = manager.nextOrder++;

		function updateClasses(): void {
			const newClasses = toClassList(computed());
			manager!.sources.set(sourceId, {
				classes: new Set(newClasses),
				order: sourceOrder,
			});
			updateElement(manager!);
			if (manager!.transitionsSuppressed) {
				manager!.transitionsSuppressed = false;
				manager!.restoreRafId = requestAnimationFrame(() => {
					manager!.restoreRafId = null;
					restoreTransitionSuppression(manager!);
				});
			}
		}

		destroyRef.onDestroy(() => {
			if (manager!.restoreRafId !== null) {
				cancelAnimationFrame(manager!.restoreRafId);
				manager!.restoreRafId = null;
			}
			if (manager!.transitionsSuppressed) {
				manager!.transitionsSuppressed = false;
				restoreTransitionSuppression(manager!);
			}
			manager!.sources.delete(sourceId);
			if (manager!.sources.size === 0) {
				cleanupManager(element);
			} else {
				updateElement(manager!);
			}
		});

		effect(updateClasses);
	});
}

function restoreTransitionSuppression(manager: ElementClassManager): void {
	const prev = manager.previousTransition;
	if (prev) {
		manager.element.style.setProperty('transition', prev, manager.previousTransitionPriority || undefined);
	} else {
		manager.element.style.removeProperty('transition');
	}
}

function setupGlobalObserver(platformId: Object): void {
	if (isPlatformBrowser(platformId) && !globalObserver) {
		globalObserver = new MutationObserver((mutations) => {
			for (const mutation of mutations) {
				if (mutation.type === 'attributes' && mutation.attributeName === 'class') {
					const element = mutation.target as HTMLElement;
					const manager = elementClassManagers.get(element);
					if (manager && observedElements.has(element)) {
						if (manager.isUpdating) continue;
						const currentClasses = toClassList(element.className);
						const allSourceClasses = new Set<string>();
						for (const source of manager.sources.values()) {
							for (const className of source.classes) {
								allSourceClasses.add(className);
							}
						}
						manager.baseClasses.clear();
						for (const className of currentClasses) {
							if (!allSourceClasses.has(className)) {
								manager.baseClasses.add(className);
							}
						}
						updateElement(manager);
					}
				}
			}
		});
		globalObserver.observe(document, {
			attributes: true,
			attributeFilter: ['class'],
			subtree: true,
		});
	}
}

function updateElement(manager: ElementClassManager): void {
	if (manager.isUpdating) return;
	manager.isUpdating = true;
	if (!manager.hasInitialized && manager.sources.size > 0) {
		const currentClasses = toClassList(manager.element.className);
		const allSourceClasses = new Set<string>();
		for (const source of manager.sources.values()) {
			source.classes.forEach((className) => allSourceClasses.add(className));
		}
		currentClasses.forEach((className) => {
			if (!allSourceClasses.has(className)) {
				manager.baseClasses.add(className);
			}
		});
		manager.hasInitialized = true;
	}
	const sortedSources = Array.from(manager.sources.entries()).sort(([, a], [, b]) => a.order - b.order);
	const allSourceClasses: string[] = [];
	for (const [, source] of sortedSources) {
		allSourceClasses.push(...source.classes);
	}
	const classesToApply =
		allSourceClasses.length > 0 || manager.baseClasses.size > 0
			? hlm([...allSourceClasses, ...manager.baseClasses])
			: '';
	if (manager.element.className !== classesToApply) {
		manager.element.className = classesToApply;
	}
	manager.isUpdating = false;
}

function cleanupManager(element: HTMLElement): void {
	observedElements.delete(element);
	elementClassManagers.delete(element);
	if (observedElements.size === 0 && globalObserver) {
		globalObserver.disconnect();
		globalObserver = null;
	}
}

interface ClassesOptions {
	elementRef?: ElementRef<HTMLElement>;
	injector?: Injector;
}

const classListCache = new Map<string, string[]>();

function toClassList(className: string | ClassValue[]): string[] {
	if (typeof className === 'string' && classListCache.has(className)) {
		return classListCache.get(className)!;
	}
	const result = clsx(className)
		.split(' ')
		.filter((c) => c.length > 0);
	if (typeof className === 'string' && classListCache.size < 1000) {
		classListCache.set(className, result);
	}
	return result;
}
