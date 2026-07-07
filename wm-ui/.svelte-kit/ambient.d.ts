
// this file is generated — do not edit it


/// <reference types="@sveltejs/kit" />

/**
 * This module provides access to environment variables that are injected _statically_ into your bundle at build time and are limited to _private_ access.
 * 
 * |         | Runtime                                                                    | Build time                                                               |
 * | ------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
 * | Private | [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private) | [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private) |
 * | Public  | [`$env/dynamic/public`](https://svelte.dev/docs/kit/$env-dynamic-public)   | [`$env/static/public`](https://svelte.dev/docs/kit/$env-static-public)   |
 * 
 * Static environment variables are [loaded by Vite](https://vitejs.dev/guide/env-and-mode.html#env-files) from `.env` files and `process.env` at build time and then statically injected into your bundle at build time, enabling optimisations like dead code elimination.
 * 
 * **_Private_ access:**
 * 
 * - This module cannot be imported into client-side code
 * - This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured)
 * 
 * For example, given the following build time environment:
 * 
 * ```env
 * ENVIRONMENT=production
 * PUBLIC_BASE_URL=http://site.com
 * ```
 * 
 * With the default `publicPrefix` and `privatePrefix`:
 * 
 * ```ts
 * import { ENVIRONMENT, PUBLIC_BASE_URL } from '$env/static/private';
 * 
 * console.log(ENVIRONMENT); // => "production"
 * console.log(PUBLIC_BASE_URL); // => throws error during build
 * ```
 * 
 * The above values will be the same _even if_ different values for `ENVIRONMENT` or `PUBLIC_BASE_URL` are set at runtime, as they are statically replaced in your code with their build time values.
 */
declare module '$env/static/private' {
	export const AGENT_GRAPHICS: string;
	export const AGENT: string;
	export const NODE_ENV: string;
	export const ALLUSERSPROFILE: string;
	export const APPDATA: string;
	export const COLOR: string;
	export const EDITOR: string;
	export const OPENCODE_CONFIG: string;
	export const GOCACHE: string;
	export const PG_INCLUDE_DIR: string;
	export const CommonProgramFiles: string;
	export const npm_config_local_prefix: string;
	export const CommonProgramW6432: string;
	export const COMPOSER_HOME: string;
	export const npm_config_userconfig: string;
	export const COMPUTERNAME: string;
	export const USERNAME: string;
	export const ComSpec: string;
	export const KIMAKI_LOCK_PORT: string;
	export const LOCALAPPDATA: string;
	export const dp0: string;
	export const DriverData: string;
	export const GOPATH: string;
	export const npm_config_global_prefix: string;
	export const HOME: string;
	export const npm_package_version: string;
	export const HOMEDRIVE: string;
	export const HOMEPATH: string;
	export const INIT_CWD: string;
	export const Path: string;
	export const npm_lifecycle_event: string;
	export const KIMAKI: string;
	export const npm_command: string;
	export const KIMAKI_DATA_DIR: string;
	export const LOGONSERVER: string;
	export const KIMAKI_DB_AUTH_TOKEN: string;
	export const OPENCODE: string;
	export const KIMAKI_DB_URL: string;
	export const npm_config_globalconfig: string;
	export const KIMAKI_OPENCODE_PROCESS: string;
	export const KIMAKI_PARENT_LOCK_PORT: string;
	export const NODE: string;
	export const npm_package_name: string;
	export const NODE_PATH: string;
	export const NVM_SYMLINK: string;
	export const npm_config_cache: string;
	export const npm_execpath: string;
	export const npm_config_node_gyp: string;
	export const npm_config_init_module: string;
	export const npm_config_noproxy: string;
	export const npm_config_npm_version: string;
	export const OPENCODE_PID: string;
	export const npm_config_prefix: string;
	export const OS: string;
	export const npm_config_user_agent: string;
	export const npm_lifecycle_script: string;
	export const OPENCODE_EXPERIMENTAL_WORKSPACES: string;
	export const npm_node_execpath: string;
	export const npm_package_json: string;
	export const NUMBER_OF_PROCESSORS: string;
	export const OPENCODE_PRINT_LOGS: string;
	export const NVM_HOME: string;
	export const OPENCHAMBER_UI_PASSWORD: string;
	export const OPENCODE_LOG_LEVEL: string;
	export const OPENCODE_PORT: string;
	export const PATHEXT: string;
	export const PGDATA: string;
	export const PROTO_APP_LOG: string;
	export const PG_LIB_DIR: string;
	export const PHP_INI_SCAN_DIR: string;
	export const PNPM_HOME: string;
	export const POWERSHELL_DISTRIBUTION_CHANNEL: string;
	export const PROCESSOR_ARCHITECTURE: string;
	export const PROCESSOR_IDENTIFIER: string;
	export const PROCESSOR_LEVEL: string;
	export const PROCESSOR_REVISION: string;
	export const ProgramData: string;
	export const ProgramFiles: string;
	export const ProgramW6432: string;
	export const PROMPT: string;
	export const PROTO_OFFLINE_TIMEOUT: string;
	export const PSModulePath: string;
	export const PROTO_SHIM_NAME: string;
	export const PROTO_SHIM_PATH: string;
	export const PROTO_VERSION: string;
	export const PUBLIC: string;
	export const STARBASE_THEME: string;
	export const SystemDrive: string;
	export const SystemRoot: string;
	export const TEMP: string;
	export const TMP: string;
	export const USERDOMAIN: string;
	export const USERDOMAIN_ROAMINGPROFILE: string;
	export const USERPROFILE: string;
	export const windir: string;
	export const __KIMAKI_CHILD: string;
	export const SVELTEKIT_FORK: string;
}

/**
 * This module provides access to environment variables that are injected _statically_ into your bundle at build time and are _publicly_ accessible.
 * 
 * |         | Runtime                                                                    | Build time                                                               |
 * | ------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
 * | Private | [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private) | [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private) |
 * | Public  | [`$env/dynamic/public`](https://svelte.dev/docs/kit/$env-dynamic-public)   | [`$env/static/public`](https://svelte.dev/docs/kit/$env-static-public)   |
 * 
 * Static environment variables are [loaded by Vite](https://vitejs.dev/guide/env-and-mode.html#env-files) from `.env` files and `process.env` at build time and then statically injected into your bundle at build time, enabling optimisations like dead code elimination.
 * 
 * **_Public_ access:**
 * 
 * - This module _can_ be imported into client-side code
 * - **Only** variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`) are included
 * 
 * For example, given the following build time environment:
 * 
 * ```env
 * ENVIRONMENT=production
 * PUBLIC_BASE_URL=http://site.com
 * ```
 * 
 * With the default `publicPrefix` and `privatePrefix`:
 * 
 * ```ts
 * import { ENVIRONMENT, PUBLIC_BASE_URL } from '$env/static/public';
 * 
 * console.log(ENVIRONMENT); // => throws error during build
 * console.log(PUBLIC_BASE_URL); // => "http://site.com"
 * ```
 * 
 * The above values will be the same _even if_ different values for `ENVIRONMENT` or `PUBLIC_BASE_URL` are set at runtime, as they are statically replaced in your code with their build time values.
 */
declare module '$env/static/public' {
	
}

/**
 * This module provides access to environment variables set _dynamically_ at runtime and that are limited to _private_ access.
 * 
 * |         | Runtime                                                                    | Build time                                                               |
 * | ------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
 * | Private | [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private) | [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private) |
 * | Public  | [`$env/dynamic/public`](https://svelte.dev/docs/kit/$env-dynamic-public)   | [`$env/static/public`](https://svelte.dev/docs/kit/$env-static-public)   |
 * 
 * Dynamic environment variables are defined by the platform you're running on. For example if you're using [`adapter-node`](https://github.com/sveltejs/kit/tree/main/packages/adapter-node) (or running [`vite preview`](https://svelte.dev/docs/kit/cli)), this is equivalent to `process.env`.
 * 
 * **_Private_ access:**
 * 
 * - This module cannot be imported into client-side code
 * - This module includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured)
 * 
 * > [!NOTE] In `dev`, `$env/dynamic` includes environment variables from `.env`. In `prod`, this behavior will depend on your adapter.
 * 
 * > [!NOTE] To get correct types, environment variables referenced in your code should be declared (for example in an `.env` file), even if they don't have a value until the app is deployed:
 * >
 * > ```env
 * > MY_FEATURE_FLAG=
 * > ```
 * >
 * > You can override `.env` values from the command line like so:
 * >
 * > ```sh
 * > MY_FEATURE_FLAG="enabled" npm run dev
 * > ```
 * 
 * For example, given the following runtime environment:
 * 
 * ```env
 * ENVIRONMENT=production
 * PUBLIC_BASE_URL=http://site.com
 * ```
 * 
 * With the default `publicPrefix` and `privatePrefix`:
 * 
 * ```ts
 * import { env } from '$env/dynamic/private';
 * 
 * console.log(env.ENVIRONMENT); // => "production"
 * console.log(env.PUBLIC_BASE_URL); // => undefined
 * ```
 */
declare module '$env/dynamic/private' {
	export const env: {
		AGENT_GRAPHICS: string;
		AGENT: string;
		NODE_ENV: string;
		ALLUSERSPROFILE: string;
		APPDATA: string;
		COLOR: string;
		EDITOR: string;
		OPENCODE_CONFIG: string;
		GOCACHE: string;
		PG_INCLUDE_DIR: string;
		CommonProgramFiles: string;
		npm_config_local_prefix: string;
		CommonProgramW6432: string;
		COMPOSER_HOME: string;
		npm_config_userconfig: string;
		COMPUTERNAME: string;
		USERNAME: string;
		ComSpec: string;
		KIMAKI_LOCK_PORT: string;
		LOCALAPPDATA: string;
		dp0: string;
		DriverData: string;
		GOPATH: string;
		npm_config_global_prefix: string;
		HOME: string;
		npm_package_version: string;
		HOMEDRIVE: string;
		HOMEPATH: string;
		INIT_CWD: string;
		Path: string;
		npm_lifecycle_event: string;
		KIMAKI: string;
		npm_command: string;
		KIMAKI_DATA_DIR: string;
		LOGONSERVER: string;
		KIMAKI_DB_AUTH_TOKEN: string;
		OPENCODE: string;
		KIMAKI_DB_URL: string;
		npm_config_globalconfig: string;
		KIMAKI_OPENCODE_PROCESS: string;
		KIMAKI_PARENT_LOCK_PORT: string;
		NODE: string;
		npm_package_name: string;
		NODE_PATH: string;
		NVM_SYMLINK: string;
		npm_config_cache: string;
		npm_execpath: string;
		npm_config_node_gyp: string;
		npm_config_init_module: string;
		npm_config_noproxy: string;
		npm_config_npm_version: string;
		OPENCODE_PID: string;
		npm_config_prefix: string;
		OS: string;
		npm_config_user_agent: string;
		npm_lifecycle_script: string;
		OPENCODE_EXPERIMENTAL_WORKSPACES: string;
		npm_node_execpath: string;
		npm_package_json: string;
		NUMBER_OF_PROCESSORS: string;
		OPENCODE_PRINT_LOGS: string;
		NVM_HOME: string;
		OPENCHAMBER_UI_PASSWORD: string;
		OPENCODE_LOG_LEVEL: string;
		OPENCODE_PORT: string;
		PATHEXT: string;
		PGDATA: string;
		PROTO_APP_LOG: string;
		PG_LIB_DIR: string;
		PHP_INI_SCAN_DIR: string;
		PNPM_HOME: string;
		POWERSHELL_DISTRIBUTION_CHANNEL: string;
		PROCESSOR_ARCHITECTURE: string;
		PROCESSOR_IDENTIFIER: string;
		PROCESSOR_LEVEL: string;
		PROCESSOR_REVISION: string;
		ProgramData: string;
		ProgramFiles: string;
		ProgramW6432: string;
		PROMPT: string;
		PROTO_OFFLINE_TIMEOUT: string;
		PSModulePath: string;
		PROTO_SHIM_NAME: string;
		PROTO_SHIM_PATH: string;
		PROTO_VERSION: string;
		PUBLIC: string;
		STARBASE_THEME: string;
		SystemDrive: string;
		SystemRoot: string;
		TEMP: string;
		TMP: string;
		USERDOMAIN: string;
		USERDOMAIN_ROAMINGPROFILE: string;
		USERPROFILE: string;
		windir: string;
		__KIMAKI_CHILD: string;
		SVELTEKIT_FORK: string;
		[key: `PUBLIC_${string}`]: undefined;
		[key: `${string}`]: string | undefined;
	}
}

/**
 * This module provides access to environment variables set _dynamically_ at runtime and that are _publicly_ accessible.
 * 
 * |         | Runtime                                                                    | Build time                                                               |
 * | ------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
 * | Private | [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private) | [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private) |
 * | Public  | [`$env/dynamic/public`](https://svelte.dev/docs/kit/$env-dynamic-public)   | [`$env/static/public`](https://svelte.dev/docs/kit/$env-static-public)   |
 * 
 * Dynamic environment variables are defined by the platform you're running on. For example if you're using [`adapter-node`](https://github.com/sveltejs/kit/tree/main/packages/adapter-node) (or running [`vite preview`](https://svelte.dev/docs/kit/cli)), this is equivalent to `process.env`.
 * 
 * **_Public_ access:**
 * 
 * - This module _can_ be imported into client-side code
 * - **Only** variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`) are included
 * 
 * > [!NOTE] In `dev`, `$env/dynamic` includes environment variables from `.env`. In `prod`, this behavior will depend on your adapter.
 * 
 * > [!NOTE] To get correct types, environment variables referenced in your code should be declared (for example in an `.env` file), even if they don't have a value until the app is deployed:
 * >
 * > ```env
 * > MY_FEATURE_FLAG=
 * > ```
 * >
 * > You can override `.env` values from the command line like so:
 * >
 * > ```sh
 * > MY_FEATURE_FLAG="enabled" npm run dev
 * > ```
 * 
 * For example, given the following runtime environment:
 * 
 * ```env
 * ENVIRONMENT=production
 * PUBLIC_BASE_URL=http://example.com
 * ```
 * 
 * With the default `publicPrefix` and `privatePrefix`:
 * 
 * ```ts
 * import { env } from '$env/dynamic/public';
 * console.log(env.ENVIRONMENT); // => undefined, not public
 * console.log(env.PUBLIC_BASE_URL); // => "http://example.com"
 * ```
 * 
 * ```
 * 
 * ```
 */
declare module '$env/dynamic/public' {
	export const env: {
		[key: `PUBLIC_${string}`]: string | undefined;
	}
}
