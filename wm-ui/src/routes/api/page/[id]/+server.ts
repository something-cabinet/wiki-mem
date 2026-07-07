import { json } from '@sveltejs/kit';
import { callTool } from '$lib/server/wm-bridge';

export async function GET({ params }) {
  const id = params.id;
  if (!id) return json({ error: 'Missing id' }, { status: 400 });
  try {
    const result = await callTool('wm_page.get', { id });
    return json(result);
  } catch (e: any) {
    return json({ error: e.message }, { status: 500 });
  }
}

export async function PUT({ params, request }) {
  const id = params.id;
  if (!id) return json({ error: 'Missing id' }, { status: 400 });
  try {
    const body = await request.json();
    const result = await callTool('wm_page.update', { id, ...body });
    return json(result);
  } catch (e: any) {
    return json({ error: e.message }, { status: 500 });
  }
}

export async function DELETE({ params }) {
  const id = params.id;
  if (!id) return json({ error: 'Missing id' }, { status: 400 });
  try {
    const result = await callTool('wm_page.delete', { id });
    return json(result);
  } catch (e: any) {
    return json({ error: e.message }, { status: 500 });
  }
}
