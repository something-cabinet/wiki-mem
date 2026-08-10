import { Injectable, signal } from '@angular/core';

type StoredTheme = 'dark' | 'light' | 'system';

@Injectable({ providedIn: 'root' })
export class ThemeService {
  private readonly media = window.matchMedia('(prefers-color-scheme: dark)');
  readonly isDark = signal(this.resolveDark());

  private storedPreference(): StoredTheme {
    const stored = localStorage.getItem('wm-dark-mode');
    if (stored === 'true') return 'dark';
    if (stored === 'false') return 'light';
    return 'system';
  }

  private resolveDark(pref: StoredTheme = this.storedPreference()): boolean {
    if (pref === 'dark') return true;
    if (pref === 'light') return false;
    return this.media.matches;
  }

  private apply(pref: StoredTheme) {
    const dark = this.resolveDark(pref);
    this.isDark.set(dark);
    document.documentElement.classList.toggle('dark', dark);
  }

  constructor() {
    this.media.addEventListener('change', () => {
      if (this.storedPreference() === 'system') {
        this.apply('system');
      }
    });
  }

  toggle() {
    const next = !this.isDark();
    localStorage.setItem('wm-dark-mode', String(next));
    this.apply(next ? 'dark' : 'light');
  }
}
