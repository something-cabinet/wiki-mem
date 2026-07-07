import { json } from '@sveltejs/kit';
import { callTool } from '$lib/server/wm-bridge';

export async function GET() {
  try {
    const result = await callTool('wm_initial');
    return json(result);
  } catch (e: any) {
    return json({ error: e.message }, { status: 500 });
  }
}
