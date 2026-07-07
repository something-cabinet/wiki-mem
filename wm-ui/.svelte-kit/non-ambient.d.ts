
// this file is generated — do not edit it


declare module "svelte/elements" {
	export interface HTMLAttributes<T> {
		'data-sveltekit-keepfocus'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-noscroll'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-preload-code'?:
			| true
			| ''
			| 'eager'
			| 'viewport'
			| 'hover'
			| 'tap'
			| 'off'
			| undefined
			| null;
		'data-sveltekit-preload-data'?: true | '' | 'hover' | 'tap' | 'off' | undefined | null;
		'data-sveltekit-reload'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-replacestate'?: true | '' | 'off' | undefined | null;
	}
}

export {};


declare module "$app/types" {
	type MatcherParam<M> = M extends (param : string) => param is (infer U extends string) ? U : string;

	export interface AppTypes {
		RouteId(): "/" | "/api" | "/api/graph" | "/api/initial" | "/api/pages" | "/api/page" | "/api/page/[id]" | "/api/rebuild" | "/api/search" | "/api/sources" | "/api/source" | "/api/tasks" | "/graph" | "/page" | "/page/[id]" | "/page/[id]/edit" | "/sources" | "/tasks";
		RouteParams(): {
			"/api/page/[id]": { id: string };
			"/page/[id]": { id: string };
			"/page/[id]/edit": { id: string }
		};
		LayoutParams(): {
			"/": { id?: string | undefined };
			"/api": { id?: string | undefined };
			"/api/graph": Record<string, never>;
			"/api/initial": Record<string, never>;
			"/api/pages": Record<string, never>;
			"/api/page": { id?: string | undefined };
			"/api/page/[id]": { id: string };
			"/api/rebuild": Record<string, never>;
			"/api/search": Record<string, never>;
			"/api/sources": Record<string, never>;
			"/api/source": Record<string, never>;
			"/api/tasks": Record<string, never>;
			"/graph": Record<string, never>;
			"/page": { id?: string | undefined };
			"/page/[id]": { id: string };
			"/page/[id]/edit": { id: string };
			"/sources": Record<string, never>;
			"/tasks": Record<string, never>
		};
		Pathname(): "/" | "/api/graph" | "/api/initial" | "/api/pages" | "/api/page" | `/api/page/${string}` & {} | "/api/rebuild" | "/api/search" | "/api/sources" | "/api/source" | "/api/tasks" | "/graph" | `/page/${string}` & {} | `/page/${string}/edit` & {} | "/sources" | "/tasks";
		ResolvedPathname(): `${"" | `/${string}`}${ReturnType<AppTypes['Pathname']>}`;
		Asset(): string & {};
	}
}