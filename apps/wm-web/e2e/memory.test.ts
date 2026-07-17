import { expect } from '@wdio/globals';

describe('WM Wiki — Memory', () => {
  beforeEach(async () => {
    await browser.url('http://localhost:4200/memory');
    await browser.pause(1000);
  });

  it('should see memory entries', async () => {
    const list = await browser.$('[role="list"]');
    await expect(list).toBeExisting();

    await expect(browser.$('body')).toHaveText(expect.stringContaining('Sync Writes Pattern'));
  });

  it('should open new memory modal', async () => {
    const newBtn = await browser.$("role=button[name='New']");
    await newBtn.click();
    await browser.pause(500);

    const dialog = await browser.$('[role="dialog"]');
    await expect(dialog).toBeExisting();
  });

  it('should create a memory entry', async () => {
    const newBtn = await browser.$("role=button[name='New']");
    await newBtn.click();
    await browser.pause(500);

    const titleInput = await browser.$("role=textbox[name='Title']");
    await titleInput.setValue('Test Memory');

    const contentInput = await browser.$("role=textbox[name='Content']");
    await contentInput.setValue('This is a test memory entry for E2E testing.');

    const tagsInput = await browser.$("role=textbox[name='Tags']");
    await tagsInput.setValue('test, e2e');

    const saveBtn = await browser.$("role=button[name='Save']");
    await saveBtn.click();
    await browser.pause(1000);
  });
});
