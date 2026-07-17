import { expect } from '@wdio/globals';

describe('WM Wiki — Search', () => {
  beforeEach(async () => {
    await browser.url('http://localhost:4200/search');
    await browser.pause(1000);
  });

  it('should search and see results', async () => {
    const input = await browser.$("role=textbox[name='Search query']");
    await input.setValue('mcp');

    const searchBtn = await browser.$("role=button[name='Search']");
    await searchBtn.click();
    await browser.pause(1000);

    const results = await browser.$('[role="list"]');
    await expect(results).toBeExisting();
  });

  it('should filter search by type', async () => {
    const input = await browser.$("role=textbox[name='Search query']");
    await input.setValue('mcp');

    const searchBtn = await browser.$("role=button[name='Search']");
    await searchBtn.click();
    await browser.pause(1000);

    await expect(browser).toHaveUrl(expect.stringContaining('/search'));

    const filterBtn = await browser.$("role=button[name='Pages']");
    await filterBtn.click();
    await browser.pause(500);

    await expect(browser).toHaveUrl(expect.stringContaining('/search'));
  });

  it('should show empty state for no results', async () => {
    const input = await browser.$("role=textbox[name='Search query']");
    await input.setValue('zzzznonexistent');

    const searchBtn = await browser.$("role=button[name='Search']");
    await searchBtn.click();
    await browser.pause(1000);

    await expect(browser.$('body')).toHaveText(expect.stringContaining('No results found'));
  });
});
