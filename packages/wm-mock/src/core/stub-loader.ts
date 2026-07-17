import type { StubMapping, ParsedStub } from './types';

export interface FileReader {
  readTextFile(path: string): string;
  listFiles(dir: string, ext: string): string[];
}

function validate(stub: unknown): boolean {
  if (!stub || typeof stub !== 'object') return false;
  const s = stub as any;
  if (!s.request || typeof s.request !== 'object') return false;
  if (typeof s.request.method !== 'string') return false;
  if (!s.request.urlPath && !s.request.urlPathPattern) return false;
  if (!s.response || typeof s.response !== 'object') return false;
  if (typeof s.response.status !== 'number') return false;
  return true;
}

export function loadStubs(fileReader: FileReader, mappingsDir: string): ParsedStub[] {
  const files = fileReader.listFiles(mappingsDir, '.json');
  const stubs: ParsedStub[] = [];
  for (const file of files) {
    const raw = fileReader.readTextFile(file);
    const parsed = JSON.parse(raw);
    if (!validate(parsed)) throw new Error(`Invalid stub file: ${file}`);
    const s = parsed as any;
    const stub: StubMapping = {
      id: pathToId(file),
      request: {
        method: s.request.method,
        urlPath: s.request.urlPath,
        urlPathPattern: s.request.urlPathPattern,
        queryParameters: s.request.queryParameters,
      },
      response: {
        status: s.response.status,
        headers: s.response.headers ?? {},
        jsonBody: s.response.jsonBody,
      },
    };
    const full: ParsedStub = { ...stub };
    if (stub.request.urlPathPattern) {
      full.urlPathRegex = new RegExp(stub.request.urlPathPattern);
    }
    stubs.push(full);
  }
  return stubs;
}

function pathToId(file: string): string {
  return file.replace(/\.json$/, '').replace(/^.*[/\\]/, '');
}
