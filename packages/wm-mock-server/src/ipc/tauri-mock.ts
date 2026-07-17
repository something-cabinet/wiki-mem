import type { MockRegistry } from '../core/registry';
import { CMD_MAP } from './cmd-map';

declare const browser: { mockIPC: (cmd: string, handler: (args: unknown) => Promise<unknown>) => Promise<void> };

export async function registerTauriMocks(registry: MockRegistry): Promise<void> {
  for (const [cmd, mapping] of Object.entries(CMD_MAP)) {
    await browser.mockIPC(cmd, async (args: unknown) => {
      const query = extractQuery((args as any)?.payload);
      const matched = registry.find(mapping.method, mapping.urlPath, query);
      if (!matched) {
        console.warn(`[mock-server] No stub for IPC "${cmd}" → ${mapping.method} ${mapping.urlPath}`);
        return undefined;
      }
      return structuredClone(matched.response.jsonBody);
    });
  }
}

function extractQuery(payload: any): Record<string, string> | undefined {
  if (!payload) return undefined;
  const query: Record<string, string> = {};
  for (const [k, v] of Object.entries(payload)) {
    if (v !== undefined && v !== null) query[k] = String(v);
  }
  return Object.keys(query).length > 0 ? query : undefined;
}
