const { I } = inject();

module.exports = {
  seeEntries() {
    I.seeElement("[role='list']");
  },
  clickNew() {
    I.click("role=button[name='New']");
  },
  fillNewForm({ title, content, tags }) {
    I.fillField("role=textbox[name='Title']", title);
    I.fillField("role=textbox[name='Content']", content);
    I.fillField("role=textbox[name='Tags']", tags);
  },
  submitNew() {
    I.click("role=button[name='Save']");
  },
  filterByLayer(layer) {
    I.selectOption("role=combobox", layer);
  },
};
