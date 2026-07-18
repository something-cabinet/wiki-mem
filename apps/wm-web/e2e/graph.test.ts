import { expect } from '@wdio/globals';
import { loadMockMappings } from './mock-helper';

describe('WM Wiki — Graph View', () => {
  before(async () => {
    await loadMockMappings();
  });

  it('should load the graph view with canvas', async () => {
    await browser.url('http://localhost:4200/graph');
    await browser.pause(2000);

    const canvas = await browser.$('canvas[wmGraph]');
    await expect(canvas).toBeExisting();

    const heading = await browser.$('h1');
    await expect(heading).toHaveText('Graph');
  });

  it('should show node and edge counts', async () => {
    await browser.url('http://localhost:4200/graph');
    await browser.pause(2000);

    const badges = await browser.$$('[wmBadge]');
    expect(badges.length).toBeGreaterThanOrEqual(2);
  });
});
