import type { MockRegistry } from '../core/registry';

/**
 * Patches global `fetch` to intercept HTTP calls matching registered stubs.
 * Returns a cleanup function to restore original fetch.
 *
 * Use in:
 * - Playwright via page.evaluate(() => installHttpInterceptor(registry))
 * - Browser dev mode as fallback
 * - Any environment where fetch is available
 */
export function installHttpInterceptor(registry: MockRegistry): () => void {
  const origFetch = window.fetch.bind(window);

  window.fetch = async (input: RequestInfo | URL, init?: RequestInit) => {
    const url = typeof input === 'string'
      ? new URL(input, window.location.origin)
      : input instanceof Request
        ? new URL(input.url)
        : new URL((input as URL).href);

    // Only intercept /api/* calls
    if (!url.pathname.startsWith('/api/')) {
      return origFetch(input, init);
    }

    const method = init?.method || 'GET';
    const matched = registry.find(method, url.pathname, Object.fromEntries(url.searchParams));

    if (!matched) {
      console.warn(`[wm-mock] No stub for ${method} ${url.pathname}`);
      return new Response(JSON.stringify({ error: 'No matching stub', path: url.pathname }), {
        status: 404,
        headers: { 'Content-Type': 'application/json' },
      });
    }

    return new Response(JSON.stringify(matched.response.jsonBody), {
      status: matched.response.status,
      headers: { 'Content-Type': 'application/json', ...matched.response.headers },
    });
  } as typeof fetch;

  return () => { window.fetch = origFetch; };
}
