import { expect } from '@wdio/globals';

describe('WM Wiki — Graph View', () => {
  before(async () => {
    await browser.url('http://localhost:4200/graph');
    await browser.pause(8000);
  });

  it('should render the canvas without loading or error states', async () => {
    const canvas = await browser.$('canvas[wmGraph]');
    await expect(canvas).toBeExisting();

    const spinner = await browser.$('wm-spinner');
    await expect(spinner).not.toBeExisting();

    const errorAlert = await browser.$('[hlmalert]');
    await expect(errorAlert).not.toBeExisting();

    const heading = await browser.$('h1');
    await expect(heading).toHaveText('Graph');
  });

  it('should show node and edge count badges', async () => {
    const badges = await browser.$$('[hlmbadge]');
    const texts = await Promise.all(badges.map((b) => b.getText()));
    const hasNodeCount = texts.some((t) => /^\d+ nodes$/.test(t));
    const hasEdgeCount = texts.some((t) => /^\d+ edges$/.test(t));
    expect(hasNodeCount).toBe(true);
    expect(hasEdgeCount).toBe(true);
  });

  it('should have interactive toolbar with zoom and spacing controls', async () => {
    const zoomIn = await browser.$('button[aria-label="Zoom in"]');
    await expect(zoomIn).toBeExisting();

    const zoomOut = await browser.$('button[aria-label="Zoom out"]');
    await expect(zoomOut).toBeExisting();

    const fitView = await browser.$('button[aria-label="Fit to view"]');
    await expect(fitView).toBeExisting();

    const slider = await browser.$('input[type="range"][aria-label="Graph node spacing"]');
    await expect(slider).toBeExisting();
    const sliderValue = await slider.getValue();
    expect(Number(sliderValue)).toBe(180);
  });

  it('should zoom in and out via toolbar buttons', async () => {
    const zoomIn = await browser.$('button[aria-label="Zoom in"]');
    await zoomIn.click();
    await browser.pause(300);
    await zoomIn.click();
    await browser.pause(300);

    const zoomOut = await browser.$('button[aria-label="Zoom out"]');
    await zoomOut.click();
    await browser.pause(300);

    const fitView = await browser.$('button[aria-label="Fit to view"]');
    await fitView.click();
    await browser.pause(500);

    const errorAlert = await browser.$('[hlmalert]');
    await expect(errorAlert).not.toBeExisting();
  });

  it('should toggle legend panel', async () => {
    const legendBtn = await browser.$('button=Legend');
    await expect(legendBtn).toBeExisting();
    await legendBtn.click();
    await browser.pause(500);

    const legendPanel = await browser.$('button=Hide');
    await expect(legendPanel).toBeExisting();

    await legendPanel.click();
    await browser.pause(300);
  });

  it('should render edge labels', async () => {
    const labels = await browser.$$('.graph-label-overlay span');
    expect(labels.length).toBeGreaterThan(0);

    const texts = await Promise.all(labels.map((l) => l.getText()));
    const edgeTypes = ['references', 'implements', 'example-of', 'part-of', 'relates-to'];
    const hasKnownTypes = texts.some((t) => edgeTypes.some((et) => t.includes(et)));
    expect(hasKnownTypes).toBe(true);
  });

  it('should adjust spacing via slider', async () => {
    const slider = await browser.$('input[type="range"][aria-label="Graph node spacing"]');
    await slider.setValue(250);
    await browser.pause(1000);
    const newValue = await slider.getValue();
    expect(Number(newValue)).toBe(250);

    await slider.setValue(180);
    await browser.pause(1000);
  });

  it('should handle pan via mouse drag on canvas', async () => {
    const canvas = await browser.$('canvas[wmGraph]');
    await canvas.waitForExist();

    const canvasEl = await canvas;
    await canvasEl.dragAndDrop({ x: 100, y: 50 });
    await browser.pause(500);

    const errorAlert = await browser.$('[hlmalert]');
    await expect(errorAlert).not.toBeExisting();
  });

  it('should update edge labels count after layout settles', async () => {
    await browser.pause(2000);

    const labels = await browser.$$('.graph-label-overlay span');
    expect(labels.length).toBeGreaterThanOrEqual(20);
  });
});
