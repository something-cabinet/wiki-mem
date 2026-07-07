import { json } from '@sveltejs/kit';
import { callTool } from '$lib/server/wm-bridge';

export async function POST({ request }) {
  try {
    const body = await request.json();
    const { action, ...args } = body;
    const result = await callTool(`wm_source.${action}`, args);
    return json(result);
  } catch (e: any) {
    return json({ error: e.message }, { status: 500 });
  }
}
