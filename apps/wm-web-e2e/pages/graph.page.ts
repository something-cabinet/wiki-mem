const { I } = inject();

module.exports = {
  seeHeading() {
    I.see("Graph", "h1");
  },
  seeStats() {
    I.see("Nodes");
    I.see("Edges");
  },
  exploreNode(nodeId) {
    I.fillField("role=textbox[name='Enter page ID...']", nodeId);
    I.click("role=button[name='Explore']");
  },
  seeNeighbors() {
    I.waitForElement('role=button', 5);
  },
};
