const { I } = inject();

module.exports = {
  seeHeading() {
    I.see("Settings", "h1");
  },
  seeEngineStatus() {
    I.see("Nodes");
    I.see("Edges");
    I.see("Uptime");
  },
  clickRefresh() {
    I.click("role=button[name='Refresh']");
  },
};
