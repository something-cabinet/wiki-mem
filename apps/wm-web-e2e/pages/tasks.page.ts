const { I } = inject();

module.exports = {
  seeHeading() {
    I.see("Task Board", "h1");
  },
  seeColumn(name) {
    I.see(name);
  },
  seeTaskCount(name, count) {
    I.see(`${name} ${count}`);
  },
};
