import Helper from '@codeceptjs/helper';

const MOCK_SERVER_URL = 'http://localhost:8081';

class MockManagerHelper extends Helper {
  async resetMocks() {
    const response = await fetch(`${MOCK_SERVER_URL}/__admin/scenarios/reset`, { method: 'POST' });
    if (!response.ok) throw new Error(`Mock reset failed: ${response.status}`);
  }

  async activateScenario(name: string) {
    const response = await fetch(`${MOCK_SERVER_URL}/__admin/scenarios/${name}/activate`, { method: 'POST' });
    if (!response.ok) throw new Error(`Scenario activation failed for "${name}": ${response.status}`);
  }

  async resetScenarios() { await this.resetMocks(); }
  async setupSessionFor(name: string) { await this.activateScenario(name); }

  async _before() { await this.resetMocks(); }

  async waitForNetworkIdle(timeout = 5000) {
    const { page } = this.helpers.Playwright;
    await page.waitForLoadState('networkidle', { timeout });
  }

  async clearBrowserState() {
    const { page } = this.helpers.Playwright;
    await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  }

  async mockRoute(urlPattern: string, responseBody: object, status = 200) {
    const { page } = this.helpers.Playwright;
    await page.route(urlPattern, (route) => {
      route.fulfill({ status, contentType: 'application/json', body: JSON.stringify(responseBody) });
    });
  }

  async unmockAllRoutes() {
    const { page } = this.helpers.Playwright;
    await page.unrouteAll({ behavior: 'wait' });
  }
}

export default MockManagerHelper;
