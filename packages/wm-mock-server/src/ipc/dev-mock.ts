import type { MockRegistry } from '../core/registry';
import { CMD_MAP } from './cmd-map';

export function createMockInvoke(registry: MockRegistry) {
  return async function mockInvoke(cmd: string, args?: Record<string, unknown>): Promise<unknown> {
    const mapping = CMD_MAP[cmd];
    if (!mapping) throw new Error(`[mock-server] Unknown IPC command: ${cmd}`);
    const query = extractQuery((args as any)?.payload);
    const matched = registry.find(mapping.method, mapping.urlPath, query);
    if (!matched) throw new Error(`[mock-server] No stub for ${mapping.urlPath} (IPC: ${cmd})`);
    return structuredClone(matched.response.jsonBody);
  };
}

function extractQuery(payload: any): Record<string, string> | undefined {
  if (!payload) return undefined;
  const query: Record<string, string> = {};
  for (const [k, v] of Object.entries(payload)) {
    if (v !== undefined && v !== null) query[k] = String(v);
  }
  return Object.keys(query).length > 0 ? query : undefined;
}
