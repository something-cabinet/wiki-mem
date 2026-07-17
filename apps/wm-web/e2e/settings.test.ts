import { expect } from '@wdio/globals';

describe('WM Wiki — Settings', () => {
  before(async () => {
    await browser.url('http://localhost:4200/settings');
    await browser.pause(1000);
  });

  it('should see settings heading', async () => {
    const h1 = await browser.$('h1');
    await expect(h1).toHaveText('Settings');
  });

  it('should see engine status with stats', async () => {
    await expect(browser.$('body')).toHaveText(expect.stringContaining('Nodes'));
    await expect(browser.$('body')).toHaveText(expect.stringContaining('Edges'));
    await expect(browser.$('body')).toHaveText(expect.stringContaining('Uptime'));
  });

  it('should refresh engine status', async () => {
    const refreshBtn = await browser.$("role=button[name='Refresh']");
    await refreshBtn.click();
    await browser.pause(1000);

    await expect(browser.$('body')).toHaveText(expect.stringContaining('Nodes'));
  });
});
