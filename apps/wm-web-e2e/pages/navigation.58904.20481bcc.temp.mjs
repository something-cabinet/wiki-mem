const { I } = inject();
const navigation = {
    goToSearch() {
        I.amOnPage("/search");
    },
    goToGraph() {
        I.amOnPage("/graph");
    },
    goToTasks() {
        I.amOnPage("/tasks");
    },
    goToPages() {
        I.amOnPage("/pages");
    },
    goToMemory() {
        I.amOnPage("/memory");
    },
    goToSettings() {
        I.amOnPage("/settings");
    },
};
export {};
