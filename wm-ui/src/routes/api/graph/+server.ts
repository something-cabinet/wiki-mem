import { json } from '@sveltejs/kit';
import { callTool } from '$lib/server/wm-bridge';

export async function GET({ url }) {
  const center = url.searchParams.get('center');
  const depth = parseInt(url.searchParams.get('depth') || '1');

  try {
    if (center) {
      const result = await callTool('wm_graph.subgraph', { center, depth });
      return json(result);
    }
    const result = await callTool('wm_graph.stats');
    return json(result);
  } catch (e: any) {
    return json({ error: e.message }, { status: 500 });
  }
}
