const { Helper } = require("codeceptjs");

class MockManager extends Helper {
  constructor(config) {
    super(config);
    this.mockUrl = config.url || "http://localhost:8081";
  }

  async resetScenarios() {
    const res = await fetch(`${this.mockUrl}/__admin/scenarios/reset`, { method: "POST" });
    if (!res.ok) {
      const text = await res.text();
      console.error(`resetScenarios failed: ${res.status} ${text}`);
    }
  }

  async setupSessionFor(name) {
    const res = await fetch(`${this.mockUrl}/__admin/scenarios/${name}/activate`, { method: "POST" });
    if (!res.ok) {
      const text = await res.text();
      throw new Error(`setupSessionFor "${name}" failed: ${res.status} ${text}`);
    }
  }
}

module.exports = MockManager;
