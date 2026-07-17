const { I } = inject();

module.exports = {
  seeList() {
    I.seeElement('[role="list"]');
  },
  clickCreate() {
    I.click("role=button[name='Create Page']");
  },
  fillCreateForm({ path, title }) {
    I.fillField("role=textbox[name='Path/ID']", path);
    I.fillField("role=textbox[name='Title']", title);
  },
  submitCreate() {
    I.click("role=button[name='Create']");
  },
  cancelCreate() {
    I.click("role=button[name='Cancel']");
  },
};
