/** WireMock-compatible stub types */

export interface StubRequest {
  method: string;
  urlPath?: string;
  urlPathPattern?: string;
  queryParameters?: Record<string, string | string[]>;
}

export interface StubResponse {
  status: number;
  headers: Record<string, string>;
  jsonBody: unknown;
}

export interface StubMapping {
  id: string;
  request: StubRequest;
  response: StubResponse;
}

export interface ParsedStub extends StubMapping {
  urlPathRegex?: RegExp;
}
