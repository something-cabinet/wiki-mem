import { Injectable, signal } from '@angular/core';

@Injectable({ providedIn: 'root' })
export class ThemeService {
  readonly isDark = signal(document.documentElement.classList.contains('dark'));

  toggle() {
    const next = !this.isDark();
    this.isDark.set(next);
    document.documentElement.classList.toggle('dark', next);
    localStorage.setItem('wm-dark-mode', String(next));
  }
}
