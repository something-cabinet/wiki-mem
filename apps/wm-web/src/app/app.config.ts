import { ApplicationConfig, provideZoneChangeDetection } from '@angular/core';
import { provideRouter } from '@angular/router';
import { routes } from './app.routes';
import { ENGINE_PORT } from './services/engine-port';
import { HttpEngineService } from './services/http-engine.service';
import { CODE_INTEL_PORT } from './services/code-intel-port';
import { HttpCodeIntelService } from './services/http-code-intel.service';

export const appConfig: ApplicationConfig = {
  providers: [
    provideZoneChangeDetection({ eventCoalescing: true }),
    provideRouter(routes),
    { provide: ENGINE_PORT, useClass: HttpEngineService },
    { provide: CODE_INTEL_PORT, useClass: HttpCodeIntelService },
  ],
};
