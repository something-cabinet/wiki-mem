import { Routes } from '@angular/router';
import { LayoutComponent } from './layout/layout.component';
import { SearchViewComponent } from './views/search/search-view.component';
import { GraphViewComponent } from './views/graph/graph-view.component';
import { TasksViewComponent } from './views/tasks/tasks-view.component';
import { PagesViewComponent } from './views/pages/pages-view.component';
import { MemoryViewComponent } from './views/memory/memory-view.component';
import { SettingsViewComponent } from './views/settings/settings-view.component';

export const routes: Routes = [
  {
    path: '',
    component: LayoutComponent,
    children: [
      { path: '', redirectTo: '/search', pathMatch: 'full' },
      { path: 'search', component: SearchViewComponent, title: 'Search' },
      { path: 'graph', component: GraphViewComponent, title: 'Graph' },
      { path: 'tasks', component: TasksViewComponent, title: 'Tasks' },
      { path: 'pages', component: PagesViewComponent, title: 'Pages' },
      { path: 'pages/:id', component: PagesViewComponent, title: 'Page' },
      { path: 'memory', component: MemoryViewComponent, title: 'Memory' },
      { path: 'settings', component: SettingsViewComponent, title: 'Settings' },
    ],
  },
];
