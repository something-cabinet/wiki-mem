export const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set([]),
	mimeTypes: {},
	_: {
		client: {start:"_app/immutable/entry/start.C3SArp9h.js",app:"_app/immutable/entry/app.C7ptl2vm.js",imports:["_app/immutable/entry/start.C3SArp9h.js","_app/immutable/chunks/DZjwGz0p.js","_app/immutable/chunks/BtMDVIzQ.js","_app/immutable/chunks/bFHyrT-9.js","_app/immutable/entry/app.C7ptl2vm.js","_app/immutable/chunks/BtMDVIzQ.js","_app/immutable/chunks/DYl5dUZ5.js","_app/immutable/chunks/xihTtKlq.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
		nodes: [
			__memo(() => import('./nodes/0.js')),
			__memo(() => import('./nodes/1.js')),
			__memo(() => import('./nodes/2.js')),
			__memo(() => import('./nodes/3.js')),
			__memo(() => import('./nodes/4.js')),
			__memo(() => import('./nodes/5.js')),
			__memo(() => import('./nodes/6.js')),
			__memo(() => import('./nodes/7.js'))
		],
		remotes: {
			
		},
		routes: [
			{
				id: "/",
				pattern: /^\/$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 2 },
				endpoint: null
			},
			{
				id: "/api/graph",
				pattern: /^\/api\/graph\/?$/,
				params: [],
				page: null,
				endpoint: __memo(() => import('./entries/endpoints/api/graph/_server.ts.js'))
			},
			{
				id: "/api/initial",
				pattern: /^\/api\/initial\/?$/,
				params: [],
				page: null,
				endpoint: __memo(() => import('./entries/endpoints/api/initial/_server.ts.js'))
			},
			{
				id: "/api/pages",
				pattern: /^\/api\/pages\/?$/,
				params: [],
				page: null,
				endpoint: __memo(() => import('./entries/endpoints/api/pages/_server.ts.js'))
			},
			{
				id: "/api/page",
				pattern: /^\/api\/page\/?$/,
				params: [],
				page: null,
				endpoint: __memo(() => import('./entries/endpoints/api/page/_server.ts.js'))
			},
			{
				id: "/api/page/[id]",
				pattern: /^\/api\/page\/([^/]+?)\/?$/,
				params: [{"name":"id","optional":false,"rest":false,"chained":false}],
				page: null,
				endpoint: __memo(() => import('./entries/endpoints/api/page/_id_/_server.ts.js'))
			},
			{
				id: "/api/rebuild",
				pattern: /^\/api\/rebuild\/?$/,
				params: [],
				page: null,
				endpoint: __memo(() => import('./entries/endpoints/api/rebuild/_server.ts.js'))
			},
			{
				id: "/api/search",
				pattern: /^\/api\/search\/?$/,
				params: [],
				page: null,
				endpoint: __memo(() => import('./entries/endpoints/api/search/_server.ts.js'))
			},
			{
				id: "/api/sources",
				pattern: /^\/api\/sources\/?$/,
				params: [],
				page: null,
				endpoint: __memo(() => import('./entries/endpoints/api/sources/_server.ts.js'))
			},
			{
				id: "/api/source",
				pattern: /^\/api\/source\/?$/,
				params: [],
				page: null,
				endpoint: __memo(() => import('./entries/endpoints/api/source/_server.ts.js'))
			},
			{
				id: "/api/tasks",
				pattern: /^\/api\/tasks\/?$/,
				params: [],
				page: null,
				endpoint: __memo(() => import('./entries/endpoints/api/tasks/_server.ts.js'))
			},
			{
				id: "/graph",
				pattern: /^\/graph\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 3 },
				endpoint: null
			},
			{
				id: "/page/[id]",
				pattern: /^\/page\/([^/]+?)\/?$/,
				params: [{"name":"id","optional":false,"rest":false,"chained":false}],
				page: { layouts: [0,], errors: [1,], leaf: 4 },
				endpoint: null
			},
			{
				id: "/page/[id]/edit",
				pattern: /^\/page\/([^/]+?)\/edit\/?$/,
				params: [{"name":"id","optional":false,"rest":false,"chained":false}],
				page: { layouts: [0,], errors: [1,], leaf: 5 },
				endpoint: null
			},
			{
				id: "/sources",
				pattern: /^\/sources\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 6 },
				endpoint: null
			},
			{
				id: "/tasks",
				pattern: /^\/tasks\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 7 },
				endpoint: null
			}
		],
		prerendered_routes: new Set([]),
		matchers: async () => {
			
			return {  };
		},
		server_assets: {}
	}
}
})();
