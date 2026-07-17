import type { ParsedStub } from './types';

export function matchStub(
  method: string,
  pathname: string,
  searchParams: URLSearchParams,
  stubs: ParsedStub[],
): ParsedStub | undefined {
  const candidates = stubs.filter((stub) => {
    if (stub.request.method !== method) return false;
    if (stub.request.urlPath) return stub.request.urlPath === pathname;
    if (stub.urlPathRegex) return stub.urlPathRegex.test(pathname);
    return false;
  });
  if (candidates.length === 0) return undefined;

  // Prefer stub with matching query parameters
  for (const stub of candidates) {
    if (!stub.request.queryParameters) continue;
    let allMatch = true;
    for (const [key, expected] of Object.entries(stub.request.queryParameters)) {
      const actual = searchParams.getAll(key);
      const expectedArr = Array.isArray(expected) ? expected : [expected];
      if (actual.length !== expectedArr.length) { allMatch = false; break; }
      if (!actual.every(v => expectedArr.includes(v))) { allMatch = false; break; }
    }
    if (allMatch) return stub;
  }
  return candidates.find(s => !s.request.queryParameters);
}
