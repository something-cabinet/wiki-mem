import { expect } from '@wdio/globals';

describe('WM Wiki — Pages', () => {
  beforeEach(async () => {
    await browser.url('http://localhost:4200/pages');
    await browser.pause(1000);
  });

  it('should see pages list', async () => {
    const list = await browser.$('[role="list"]');
    await expect(list).toBeExisting();

    await expect(browser.$('body')).toHaveText(expect.stringContaining('Knowns Reference'));
    await expect(browser.$('body')).toHaveText(expect.stringContaining('Graph Architecture'));
  });

  it('should open create page modal', async () => {
    const createBtn = await browser.$("role=button[name='Create Page']");
    await createBtn.click();
    await browser.pause(500);

    const dialog = await browser.$('[role="dialog"]');
    await expect(dialog).toBeExisting();

    const cancelBtn = await browser.$("role=button[name='Cancel']");
    await cancelBtn.click();
  });

  it('should create a new page', async () => {
    const createBtn = await browser.$("role=button[name='Create Page']");
    await createBtn.click();
    await browser.pause(500);

    const pathInput = await browser.$("role=textbox[name='Path/ID']");
    await pathInput.setValue('concepts/test-page');

    const titleInput = await browser.$("role=textbox[name='Title']");
    await titleInput.setValue('Test Page');

    const submitBtn = await browser.$("role=button[name='Create']");
    await submitBtn.click();
    await browser.pause(1000);
  });
});
