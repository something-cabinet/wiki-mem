import { expect } from '@wdio/globals';

describe('WM Wiki — Tasks', () => {
  before(async () => {
    await browser.url('http://localhost:4200/tasks');
    await browser.pause(1000);
  });

  it('should see task board heading', async () => {
    const h1 = await browser.$('h1');
    await expect(h1).toHaveText('Task Board');
  });

  it('should see task board columns', async () => {
    await expect(browser.$('body')).toHaveText(expect.stringContaining('Todo'));
    await expect(browser.$('body')).toHaveText(expect.stringContaining('Done'));
  });

  it('should see existing tasks in Todo column', async () => {
    await expect(browser.$('body')).toHaveText(expect.stringContaining('Add ServerHandler impl'));
    await expect(browser.$('body')).toHaveText(expect.stringContaining('Refactor wm-cli'));
    await expect(browser.$('body')).toHaveText(expect.stringContaining('Refactor wm-server'));
  });
});
