import { json } from '@sveltejs/kit';
import { callTool } from '$lib/server/wm-bridge';

export async function GET({ url }) {
  const q = url.searchParams.get('q') || '';
  const mode = url.searchParams.get('mode') || 'hybrid';
  const limit = parseInt(url.searchParams.get('limit') || '20');

  try {
    const result = await callTool('wm_search.query', { q, mode, limit });
    return json(result);
  } catch (e: any) {
    return json({ error: e.message }, { status: 500 });
  }
}
