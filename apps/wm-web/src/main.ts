import { bootstrapApplication } from '@angular/platform-browser';
import { appConfig } from './app/app.config';
import { AppComponent } from './app/app.component';

// ─── Mock mode for tauri dev ──────────────────────
// Start the app with ?mock=true to use fake data instead of real IPC.
async function main() {
  if (typeof window !== 'undefined' && window.location.search.includes('mock=true')) {
    const { MockRegistry, createMockInvoke } = await import('@vpp-rag/mock');

    // Make invoke() available via the browser context
    // Instead of loading stubs from the filesystem, we inline a minimal set
    // for prototyping. The full stub loader can be added when mappings/ are
    // bundled as assets.
    const registry = new MockRegistry();
    (window as any).__TAURI_INTERNALS__ = {};
    (window as any).__MOCK_INVOKE__ = createMockInvoke(registry);
  }

  bootstrapApplication(AppComponent, appConfig).catch((err) =>
    console.error(err),
  );
}

main();
