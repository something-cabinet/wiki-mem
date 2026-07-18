import { Directive } from '@angular/core';
import { classes } from '@ui/utils';

@Directive({
	selector: 'main[hlmSidebarInset]',
	host: { 'data-slot': 'sidebar-inset' },
})
export class HlmSidebarInset {
	constructor() {
		classes(() => 'spartan-sidebar-inset relative flex w-full flex-1 flex-col');
	}
}
