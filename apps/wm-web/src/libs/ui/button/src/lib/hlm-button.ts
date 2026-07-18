import { Directive, input, signal } from '@angular/core';
import { BrnButton } from '@spartan-ng/brain/button';
import { classes } from '@ui/utils';
import { cva, type VariantProps } from 'class-variance-authority';
import type { ClassValue } from 'clsx';
import { injectBrnButtonConfig } from './hlm-button.token';

export const buttonVariants = cva(
	'group/button inline-flex shrink-0 items-center justify-center whitespace-nowrap transition-all outline-none select-none data-disabled:pointer-events-none data-disabled:opacity-50 [&_ng-icon]:pointer-events-none [&_ng-icon]:shrink-0',
	{
		variants: {
			variant: {
				default: 'bg-primary text-primary-foreground hover:bg-primary/90 shadow-sm',
				outline: 'border border-input bg-background hover:bg-accent hover:text-accent-foreground shadow-sm',
				secondary: 'bg-secondary text-secondary-foreground hover:bg-secondary/80 shadow-sm',
				ghost: 'text-foreground hover:bg-accent hover:text-accent-foreground',
				destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/90 shadow-sm',
				link: 'text-primary underline-offset-4 hover:underline',
			},
			size: {
				default: 'h-10 px-4 py-2 text-sm rounded-lg',
				xs: 'h-7 px-2 text-xs rounded-md',
				sm: 'h-8 px-3 text-xs rounded-md',
				lg: 'h-12 px-6 text-base rounded-lg',
				icon: 'h-10 w-10 rounded-lg',
				'icon-xs': 'h-7 w-7 rounded-md',
				'icon-sm': 'h-8 w-8 rounded-md',
				'icon-lg': 'h-12 w-12 rounded-lg',
			},
		},
		defaultVariants: {
			variant: 'default',
			size: 'default',
		},
	},
);

export type ButtonVariants = VariantProps<typeof buttonVariants>;

@Directive({
	selector: 'button[hlmBtn], a[hlmBtn]',
	exportAs: 'hlmBtn',
	hostDirectives: [{ directive: BrnButton, inputs: ['disabled'] }],
	host: { 'data-slot': 'button' },
})
export class HlmButton {
	private readonly _config = injectBrnButtonConfig();
	private readonly _additionalClasses = signal<ClassValue>('');
	public readonly variant = input<ButtonVariants['variant']>(this._config.variant);
	public readonly size = input<ButtonVariants['size']>(this._config.size);

	constructor() {
		classes(() => [buttonVariants({ variant: this.variant(), size: this.size() }), this._additionalClasses()]);
	}

	setClass(classes: string): void {
		this._additionalClasses.set(classes);
	}
}
