import type { MockRegistry } from '../core/registry';
import { CMD_MAP } from '../core/cmd-map';

export function createMockInvoke(registry: MockRegistry) {
  return async function mockInvoke(cmd: string, args?: Record<string, unknown>): Promise<unknown> {
    const mapping = CMD_MAP[cmd];
    if (!mapping) {
      throw new Error(`[wm-mock] Unknown IPC command: ${cmd}`);
    }
    const query: Record<string, string> | undefined = (args as any)?.payload
      ? Object.fromEntries(
          Object.entries((args as any).payload)
            .filter(([, v]: [string, any]) => v !== undefined && v !== null)
            .map(([k, v]: [string, any]) => [k, String(v)]),
        )
      : undefined;
    const matched = registry.find(mapping.method, mapping.urlPath, query);
    if (!matched) {
      throw new Error(`[wm-mock] No stub for ${mapping.method} ${mapping.urlPath} (IPC: ${cmd})`);
    }
    return structuredClone(matched.response.jsonBody);
  };
}
