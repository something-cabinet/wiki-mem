import { expect } from '@wdio/globals';

const routes = ['/search', '/graph', '/tasks', '/pages', '/memory', '/settings'] as const;
const linkNames = ['Search', 'Graph', 'Tasks', 'Pages', 'Memory', 'Settings'] as const;

describe('WM Wiki — Navigation', () => {
  before(async () => {
    await browser.url('http://localhost:4200');
    await browser.pause(1000);
  });

  for (let i = 0; i < routes.length; i++) {
    it(`should navigate to ${routes[i]} via sidebar link`, async () => {
      const link = await browser.$(`role=link[name="${linkNames[i]}"]`);
      await link.click();
      await browser.pause(500);
      await expect(browser).toHaveUrl(expect.stringContaining(routes[i]));
    });
  }
});
