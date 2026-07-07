import { json } from '@sveltejs/kit';
import { callTool } from '$lib/server/wm-bridge';

export async function POST({ request }) {
  try {
    const body = await request.json();
    const { path, title, content, type } = body;
    const result = await callTool('wm_page.create', { path, title, content, type });
    return json(result);
  } catch (e: any) {
    return json({ error: e.message }, { status: 500 });
  }
}
