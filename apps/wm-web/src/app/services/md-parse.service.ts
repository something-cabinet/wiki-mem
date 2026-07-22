import { Injectable } from '@angular/core';

export interface ParsedDoc {
  frontmatter: string;
  body: string;
}

@Injectable({ providedIn: 'root' })
export class MdParseService {
  private wasm: any = null;
  private loaded = false;

  async load(): Promise<void> {
    if (this.loaded) return;
    const wasmModule = await import('../../assets/wasm/md-parse/md_parse_wasm.js');
    await wasmModule.default();
    this.wasm = wasmModule;
    this.loaded = true;
  }

  parseMarkdown(text: string): ParsedDoc {
    const result = this.wasm?.parse_markdown(text);
    return result ? JSON.parse(result) : { frontmatter: '', body: text };
  }

  parseFrontmatter(text: string): Record<string, string> {
    const result = this.wasm?.parse_frontmatter(text);
    return result ? JSON.parse(result) : {};
  }
}
