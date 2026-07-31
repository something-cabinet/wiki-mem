use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use ratatui::symbols::border::Set as BorderSet;
use ratatui::widgets::*;
use ratatui::Frame;

use petgraph::visit::EdgeRef;
use wm_core::engine::MainEngine;
use wm_core::page::get_page;

fn use_unicode() -> bool {
    if std::env::var("NO_UNICODE").is_ok() {
        return false;
    }
    if let Ok(term) = std::env::var("TERM") {
        if term == "linux" || term == "dumb" {
            return false;
        }
    }
    true
}

const ASCII_BORDER: BorderSet = BorderSet {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    horizontal_top: "-",
    horizontal_bottom: "-",
    vertical_left: "|",
    vertical_right: "|",
};

fn block_bordered(title: impl Into<Line<'static>>) -> Block<'static> {
    if use_unicode() {
        Block::bordered().title(title)
    } else {
        Block::default()
            .borders(Borders::ALL)
            .border_set(ASCII_BORDER)
            .title(title)
    }
}

fn paste_from_clipboard() -> Option<String> {
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", "Get-Clipboard"])
        .output()
        .ok()?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    } else {
        None
    }
}

pub fn run_tui(engine: Arc<MainEngine>) -> Result<(), anyhow::Error> {
    enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

    let mut app = App::new(engine);
    let result = run_event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    std::io::stdout().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<(), anyhow::Error> {
    loop {
        terminal.draw(|f| app.render(f))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if app.show_help && handle_help_key(app, key.code) != KeyAction::NotHandled {
                    continue;
                }

                if app.preview_content.is_some()
                    && handle_preview_key(app, key.code) != KeyAction::NotHandled
                {
                    continue;
                }

                match handle_navigation_key(app, key.code) {
                    KeyAction::Quit => return Ok(()),
                    KeyAction::Handled => continue,
                    KeyAction::NotHandled => {}
                }

                handle_action_key(app, key.code);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum KeyAction {
    Handled,
    Quit,
    NotHandled,
}

fn handle_help_key(app: &mut App, code: KeyCode) -> KeyAction {
    match code {
        KeyCode::Char('?') | KeyCode::Esc => {
            app.show_help = false;
            KeyAction::Handled
        }
        _ => KeyAction::Handled,
    }
}

fn handle_preview_key(app: &mut App, code: KeyCode) -> KeyAction {
    match code {
        KeyCode::Esc => {
            app.preview_content = None;
            app.preview_id = None;
            app.preview_scroll = 0;
            KeyAction::Handled
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.preview_scroll = app.preview_scroll.saturating_sub(1);
            KeyAction::Handled
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.preview_scroll = app.preview_scroll.saturating_add(1);
            KeyAction::Handled
        }
        KeyCode::PageUp => {
            app.preview_scroll = app.preview_scroll.saturating_sub(10);
            KeyAction::Handled
        }
        KeyCode::PageDown => {
            app.preview_scroll = app.preview_scroll.saturating_add(10);
            KeyAction::Handled
        }
        _ => KeyAction::Handled,
    }
}

fn handle_navigation_key(app: &mut App, code: KeyCode) -> KeyAction {
    match code {
        KeyCode::Char('q') => return KeyAction::Quit,
        KeyCode::Char('s') => {
            app.active_tab = Tab::Search;
            app.input_mode = InputMode::Query;
        }
        KeyCode::Char('d') => app.active_tab = Tab::Dashboard,
        KeyCode::Char('g') => app.active_tab = Tab::Graph,
        KeyCode::Char('t') => app.active_tab = Tab::Tasks,
        KeyCode::Char('h') => app.active_tab = Tab::Help,
        KeyCode::Char('?') => app.show_help = true,
        KeyCode::Tab => {
            app.active_tab = match app.active_tab {
                Tab::Help => Tab::Search,
                Tab::Search => Tab::Graph,
                Tab::Graph => Tab::Dashboard,
                Tab::Dashboard => Tab::Tasks,
                Tab::Tasks => Tab::Help,
            };
        }
        KeyCode::BackTab => {
            app.active_tab = match app.active_tab {
                Tab::Help => Tab::Tasks,
                Tab::Tasks => Tab::Dashboard,
                Tab::Dashboard => Tab::Graph,
                Tab::Graph => Tab::Search,
                Tab::Search => Tab::Help,
            };
        }
        KeyCode::Up | KeyCode::Char('k') => match app.active_tab {
            Tab::Dashboard if app.dashboard_scroll > 0 => {
                app.dashboard_scroll = app.dashboard_scroll.saturating_sub(1);
            }
            Tab::Search if app.input_mode == InputMode::Results && app.list_index > 0 => {
                app.list_index = app.list_index.saturating_sub(1);
            }
            _ => {}
        },
        KeyCode::Down | KeyCode::Char('j') => match app.active_tab {
            Tab::Dashboard => {
                let snapshot = app.engine.state.graph.load();
                let total = snapshot.0.node_count();
                if app.dashboard_scroll.saturating_add(1) < total {
                    app.dashboard_scroll = app.dashboard_scroll.saturating_add(1);
                }
            }
            Tab::Search if app.input_mode == InputMode::Results => {
                app.list_index = app
                    .list_index
                    .saturating_add(1)
                    .min(app.search_results.len().saturating_sub(1));
            }
            _ => {}
        },
        KeyCode::PageUp if app.active_tab == Tab::Dashboard => {
            app.dashboard_scroll = app.dashboard_scroll.saturating_sub(10);
        }
        KeyCode::PageDown if app.active_tab == Tab::Dashboard => {
            let snapshot = app.engine.state.graph.load();
            let total = snapshot.0.node_count();
            app.dashboard_scroll =
                (app.dashboard_scroll.saturating_add(10)).min(total.saturating_sub(1));
        }
        _ => return KeyAction::NotHandled,
    }
    KeyAction::Handled
}

fn handle_action_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('i') if app.active_tab == Tab::Search => {
            app.input_mode = InputMode::Query;
        }
        KeyCode::Char('\x16')
            if app.active_tab == Tab::Search && app.input_mode == InputMode::Query =>
        {
            if let Some(text) = paste_from_clipboard() {
                app.search_query.push_str(&text);
            }
        }
        KeyCode::Char(c) if app.active_tab == Tab::Search && app.input_mode == InputMode::Query => {
            app.search_query.push(c);
        }
        KeyCode::Backspace
            if app.active_tab == Tab::Search && app.input_mode == InputMode::Query =>
        {
            app.search_query.pop();
        }
        KeyCode::Enter if app.active_tab == Tab::Search => {
            if app.input_mode == InputMode::Results && !app.search_results.is_empty() {
                if let Some(result) = app.search_results.get(app.list_index) {
                    match get_page(&app.engine.state, &result.id) {
                        Ok(content) => {
                            app.preview_id = Some(result.id.clone());
                            app.preview_content = Some(content.raw);
                            app.graph_center = Some(result.id.clone());
                        }
                        Err(e) => {
                            app.status = format!("Preview error: {}", e);
                        }
                    }
                }
            } else {
                app.run_search();
            }
        }
        _ => {}
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Dashboard,
    Search,
    Graph,
    Tasks,
    Help,
}

#[derive(Clone, Copy, PartialEq)]
enum InputMode {
    Query,
    Results,
}

struct SearchResult {
    id: String,
    score: f64,
    snippet: String,
}

struct App {
    engine: Arc<MainEngine>,
    active_tab: Tab,
    input_mode: InputMode,
    list_index: usize,
    status: String,
    search_query: String,
    search_results: Vec<SearchResult>,
    dashboard_scroll: usize,
    search_scroll: usize,
    preview_content: Option<String>,
    preview_id: Option<String>,
    preview_scroll: usize,
    show_help: bool,
    graph_center: Option<String>,
}

impl App {
    fn new(engine: Arc<MainEngine>) -> Self {
        Self {
            engine,
            active_tab: Tab::Dashboard,
            input_mode: InputMode::Query,
            list_index: 0,
            status: "h/d/s/g/t: tab  Tab/Shift+Tab: cycle  ?: help  q: quit".into(),
            search_query: String::new(),
            search_results: Vec::new(),
            dashboard_scroll: 0,
            search_scroll: 0,
            preview_content: None,
            preview_id: None,
            preview_scroll: 0,
            show_help: false,
            graph_center: None,
        }
    }

    fn run_search(&mut self) {
        if self.search_query.trim().is_empty() {
            self.status = "Enter a query first".into();
            return;
        }
        let bm25 = self.engine.state.bm25_index.load();
        let results = bm25.search(&self.search_query, 20);
        self.search_results = results
            .iter()
            .map(|r| SearchResult {
                id: r.id.clone(),
                score: r.score,
                snippet: r.snippet.clone(),
            })
            .collect();
        self.list_index = 0;
        self.input_mode = InputMode::Results;
        self.status = format!(
            "{} results for '{}'",
            self.search_results.len(),
            self.search_query
        );
    }

    fn render(&mut self, f: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(f.size());

        let tab_titles = ["Dashboard", "Search", "Graph", "Tasks", "Help"]
            .iter()
            .enumerate()
            .map(|(i, title)| {
                let is_active = match self.active_tab {
                    Tab::Dashboard => i == 0,
                    Tab::Search => i == 1,
                    Tab::Graph => i == 2,
                    Tab::Tasks => i == 3,
                    Tab::Help => i == 4,
                };
                let prefix = if is_active { " > " } else { "   " };
                Line::from(format!("{}{}", prefix, title))
            })
            .collect::<Vec<_>>();

        let tabs = Tabs::new(tab_titles)
            .block(block_bordered(" Wiki Memory Engine "))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(tabs, layout[0]);

        match self.active_tab {
            Tab::Dashboard => self.render_dashboard(f, layout[1]),
            Tab::Search => self.render_search(f, layout[1]),
            Tab::Graph => self.render_graph(f, layout[1]),
            Tab::Tasks => self.render_tasks(f, layout[1]),
            Tab::Help => self.render_help_tab(f, layout[1]),
        }

        let mut status_text = self.status.clone();
        if self.active_tab == Tab::Dashboard {
            let snapshot = self.engine.state.graph.load();
            let total = snapshot.0.node_count();
            let pos = self.dashboard_scroll.saturating_add(1);
            if total > 50 {
                status_text.push_str(&format!("  [{}/{}]", pos.min(total), total));
            }
        }
        if self.show_help {
            status_text = "Press ? or Esc to close help overlay".into();
        } else if self.active_tab == Tab::Help {
            status_text = "Help tab — ? for overlay, h/tab to navigate away".into();
        }
        if self.preview_content.is_some() {
            status_text = format!(
                "Preview: {} — Esc to close",
                self.preview_id.as_deref().unwrap_or("")
            );
        }
        let status = Paragraph::new(Text::from(status_text.as_str()))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(status, layout[2]);

        if self.show_help {
            self.render_help(f);
        }
    }

    fn render_help(&self, f: &mut Frame) {
        let area = f.size();
        let help_area = Rect {
            x: area.width / 6,
            y: area.height / 4,
            width: area.width.saturating_mul(2) / 3,
            height: area.height / 2,
        };
        let (arrow_r, arrow_u, arrow_d) = if use_unicode() {
            ("\u{2192}", "\u{2191}", "\u{2193}")
        } else {
            ("->", "^", "v")
        };
        let bindings: Vec<String> = vec![
            "q                    Quit".into(),
            "h / d / s / g / t   Switch to Help / Dashboard / Search / Graph / Tasks".into(),
            format!(
                "Tab                  Cycle tab forward  (help{}search{}graph{}dashboard{}tasks)",
                arrow_r, arrow_r, arrow_r, arrow_r
            ),
            format!(
                "Shift+Tab            Cycle tab backward  (tasks{}dashboard{}graph{}search{}help)",
                arrow_r, arrow_r, arrow_r, arrow_r
            ),
            format!(
                "{}/k  {}/j              Navigate list / scroll",
                arrow_u, arrow_d
            ),
            "Enter                Search / preview result".into(),
            "i (Search tab)       Focus query input".into(),
            "Ctrl+V               Paste from clipboard (Search query)".into(),
            "?                    Toggle this help overlay".into(),
        ];
        let content = Paragraph::new(Text::from(bindings.join("\n")))
            .block(block_bordered(" Help "))
            .style(Style::default())
            .alignment(Alignment::Left);
        f.render_widget(Clear, help_area);
        f.render_widget(content, help_area);
    }

    fn render_help_tab(&self, f: &mut Frame, area: Rect) {
        let (arrow_u, arrow_d) = if use_unicode() {
            ("\u{2191}", "\u{2193}")
        } else {
            ("^", "v")
        };
        let bindings: Vec<(&str, Vec<String>)> = vec![
            (
                "General",
                vec![
                    "q       Quit".into(),
                    "?       Toggle help overlay (on top of current tab)".into(),
                ],
            ),
            (
                "Navigation",
                vec![
                    "Tab     Cycle tab forward".into(),
                    "Shift+Tab  Cycle tab backward".into(),
                    "h/d/s/g/t  Switch to Help / Dashboard / Search / Graph / Tasks".into(),
                ],
            ),
            (
                "Dashboard",
                vec![format!("{}/k  {}/j  Scroll page list", arrow_u, arrow_d)],
            ),
            (
                "Search",
                vec![
                    "i       Focus query input".into(),
                    "Enter   Run search / preview result".into(),
                    "Ctrl+V  Paste from clipboard".into(),
                    format!("{}/k  {}/j  Navigate results", arrow_u, arrow_d),
                    "Esc     Close preview".into(),
                ],
            ),
            (
                "Graph",
                vec!["(read-only view of centered node and neighbors)".into()],
            ),
        ];
        let mut content = String::new();
        for (section, keys) in &bindings {
            content.push_str(&format!("{}\n", section));
            for k in keys {
                content.push_str(&format!("  {}\n", k));
            }
            content.push('\n');
        }
        let title: String = if use_unicode() {
            " Help \u{2014} Key Bindings ".into()
        } else {
            " Help - Key Bindings ".into()
        };
        let widget = Paragraph::new(Text::from(content))
            .block(block_bordered(title))
            .style(Style::default());
        f.render_widget(widget, area);
    }

    fn render_dashboard(&mut self, f: &mut Frame, area: Rect) {
        let snapshot = self.engine.state.graph.load();
        let graph = &snapshot.0;
        let node_count = graph.node_count();
        let edge_count = graph.edge_count();
        let sections = self.engine.state.section_corpus.load().len();
        let bm25_docs = self.engine.state.bm25_index.load().total_docs;

        let mut page_types: std::collections::BTreeMap<String, usize> =
            std::collections::BTreeMap::new();
        for idx in graph.node_indices() {
            let type_name = format!("{:?}", graph[idx].page_type).to_lowercase();
            let count = page_types.entry(type_name).or_insert(0);
            *count = count.wrapping_add(1);
        }

        let header = format!(
            "Nodes: {}  Edges: {}  Sections: {}  BM25: {}\n",
            node_count, edge_count, sections, bm25_docs
        );
        let types: String = page_types
            .iter()
            .map(|(t, c)| format!("  {}: {}\n", t, c))
            .collect();

        if node_count > 50 {
            let inner = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(area);

            let max_scroll = node_count.saturating_sub(1);
            if self.dashboard_scroll > max_scroll {
                self.dashboard_scroll = max_scroll;
            }

            let mut items: Vec<ListItem> = vec![ListItem::new(Line::from(format!(
                "Nodes: {}  Edges: {}  Sections: {}  BM25: {}",
                node_count, edge_count, sections, bm25_docs
            )))];
            for (t, c) in &page_types {
                items.push(ListItem::new(Line::from(format!("  {}: {}", t, c))));
            }
            items.push(ListItem::new(Line::from("")));
            items.push(ListItem::new(Line::from("Pages:")));
            for idx in graph.node_indices() {
                let meta = &graph[idx];
                items.push(ListItem::new(Line::from(format!(
                    "  {} [{}]",
                    meta.title, meta.id
                ))));
            }

            let total_items = items.len();

            let window_size = usize::from(inner[0].height).saturating_sub(2);
            let window_size = window_size.max(1);
            let start = self.dashboard_scroll.min(total_items.saturating_sub(1));
            let end = (start.saturating_add(window_size)).min(total_items);

            let visible_items: Vec<ListItem> = items
                .into_iter()
                .skip(start)
                .take(end.saturating_sub(start))
                .collect();

            let list = List::new(visible_items)
                .block(block_bordered(" Dashboard "))
                .style(Style::default());
            f.render_widget(list, inner[0]);

            let mut scroll_state = ScrollbarState::new(total_items).position(self.dashboard_scroll);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            f.render_stateful_widget(scrollbar, inner[1], &mut scroll_state);
        } else {
            let pages: String = graph
                .node_indices()
                .map(|idx| {
                    let meta = &graph[idx];
                    format!("  {} [{}]\n", meta.title, meta.id)
                })
                .collect();

            let content = format!("{}{}\nPages:\n{}", header, types, pages);
            let stats = Paragraph::new(Text::from(content))
                .block(block_bordered(" Dashboard "))
                .style(Style::default());
            f.render_widget(stats, area);
        }
    }

    fn render_search(&mut self, f: &mut Frame, area: Rect) {
        if let Some(ref content) = self.preview_content {
            let total_lines = content.lines().count();
            let max_lines = usize::from(area.height).saturating_sub(2).max(1);

            let max_scroll = total_lines.saturating_sub(max_lines);
            if self.preview_scroll > max_scroll {
                self.preview_scroll = max_scroll;
            }

            let lines: Vec<&str> = content.lines().collect();
            let start = self.preview_scroll;
            let end = (start.saturating_add(max_lines)).min(total_lines);
            let visible: Vec<Line> = lines[start..end]
                .iter()
                .map(|l| Line::from(l.to_string()))
                .collect();

            let preview_title = format!(" Preview: {} ", self.preview_id.as_deref().unwrap_or(""));
            let preview = Paragraph::new(Text::from(visible))
                .block(block_bordered(preview_title))
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(preview, area);
            return;
        }

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        let query_style = if self.input_mode == InputMode::Query {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };
        let input = Paragraph::new(Text::from(format!("> {}", self.search_query)))
            .block(block_bordered(" Query "))
            .style(query_style);
        f.render_widget(input, layout[0]);

        if self.search_results.is_empty() {
            let empty = Paragraph::new(Text::from("Type a query and press Enter to search."))
                .block(block_bordered(" Results "));
            f.render_widget(empty, layout[1]);
            return;
        }

        let items: Vec<ListItem> = self
            .search_results
            .iter()
            .map(|r| {
                let line = format!("{}%  {}  {}", (r.score * 100.0) as u8, r.id, r.snippet);
                ListItem::new(Line::from(line))
            })
            .collect();

        let total_items = items.len();

        let inner = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(layout[1]);

        let window_size = usize::from(inner[0].height).saturating_sub(2).max(1);

        if self.list_index < self.search_scroll {
            self.search_scroll = self.list_index;
        }
        if self.list_index >= self.search_scroll.saturating_add(window_size) {
            self.search_scroll = self
                .list_index
                .saturating_sub(window_size)
                .saturating_add(1);
        }

        let start = self.search_scroll.min(total_items.saturating_sub(1));
        let end = (start.saturating_add(window_size)).min(total_items);

        let visible_items: Vec<ListItem> = items
            .into_iter()
            .skip(start)
            .take(end.saturating_sub(start))
            .collect();

        let highlight_symbol = if use_unicode() { " \u{2192} " } else { " -> " };

        let list = List::new(visible_items)
            .block(block_bordered(format!(" Results ({}) ", total_items)))
            .highlight_style(Style::default().fg(Color::Cyan))
            .highlight_symbol(highlight_symbol)
            .style(Style::default());

        let local_selected = self.list_index.checked_sub(start);
        let mut list_state = ListState::default();
        list_state.select(local_selected);
        f.render_stateful_widget(list, inner[0], &mut list_state);

        let mut scroll_state = ScrollbarState::new(total_items).position(self.search_scroll);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        f.render_stateful_widget(scrollbar, inner[1], &mut scroll_state);
    }

    fn render_graph(&self, f: &mut Frame, area: Rect) {
        let snapshot = self.engine.state.graph.load();
        let graph = &snapshot.0;
        let unicode = use_unicode();

        let center_id = self.graph_center.as_deref();
        let content = if graph.node_count() == 0 {
            "No pages in wiki.".into()
        } else {
            let center_idx: Option<petgraph::graph::NodeIndex> = center_id
                .and_then(|id| snapshot.1.get(id).copied())
                .or_else(|| graph.node_indices().next());

            match center_idx {
                Some(start) => {
                    let meta = &graph[start];
                    let mut lines = format!("Center: {} [{}]\n\nNeighbors:\n", meta.title, meta.id);
                    let (edge_pre, edge_post) = if unicode {
                        ("\u{2500}\u{2500}", "\u{2500}\u{2500}\u{25b6}")
                    } else {
                        ("--", "-->")
                    };
                    for edge in graph.edges(start) {
                        let target = edge.target();
                        let t = &graph[target];
                        let e = format!("{:?}", edge.weight()).to_lowercase();
                        lines.push_str(&format!(
                            "  {}{}{} {} [{}]\n",
                            edge_pre, e, edge_post, t.title, t.id
                        ));
                    }
                    if graph.edges(start).count() == 0 {
                        lines.push_str("  (no edges)\n");
                    }
                    let incoming: Vec<_> = graph
                        .edges_directed(start, petgraph::Direction::Incoming)
                        .collect();
                    if !incoming.is_empty() {
                        lines.push_str("\nIncoming:\n");
                        for edge in incoming {
                            let source = edge.source();
                            let s = &graph[source];
                            let e = format!("{:?}", edge.weight()).to_lowercase();
                            lines.push_str(&format!(
                                "  {} [{}] {}{}{}\n",
                                s.title, s.id, edge_pre, e, edge_post
                            ));
                        }
                    }
                    lines
                }
                None => "No pages in wiki.".into(),
            }
        };

        let title = if unicode {
            format!(" Graph \u{2014} {} ", center_id.unwrap_or(""))
        } else {
            format!(" Graph -- {} ", center_id.unwrap_or(""))
        };
        let stats = Paragraph::new(Text::from(content))
            .block(block_bordered(title))
            .style(Style::default());
        f.render_widget(stats, area);
    }

    fn render_tasks(&self, f: &mut Frame, area: Rect) {
        let board = wm_core::task::build_task_board(&self.engine.state);
        let unicode = use_unicode();

        let column_order = [
            "draft",
            "todo",
            "in-progress",
            "in-review",
            "blocked",
            "done",
            "reviewed",
            "approved",
            "superseded",
            "cancelled",
        ];
        let markers = if unicode {
            [
                "\u{25a1}", "\u{25d0}", "\u{25d0}", "\u{25d0}", "\u{26a0}", "\u{2713}", "\u{2713}",
                "\u{2713}", "\u{2713}", "\u{2717}",
            ]
        } else {
            [
                "[ ]", "[-]", "[-]", "[-]", "[!]", "[x]", "[x]", "[x]", "[x]", "[X]",
            ]
        };

        let mut parts: Vec<String> = Vec::new();
        for (i, col_name) in column_order.iter().enumerate() {
            let items = board.columns.get(*col_name);
            let count = items.map(|v| v.len()).unwrap_or(0);
            if count == 0 {
                continue;
            }
            let label = col_name.to_uppercase().replace('-', " ");
            let marker = markers[i];
            let items_str = items
                .expect("items should exist when count > 0")
                .iter()
                .map(|t| format!("  {} {}\n", marker, t.title))
                .collect::<String>();
            parts.push(format!("{} ({}):\n{}", label, count, items_str));
        }

        let content = if parts.is_empty() {
            "(no tasks)".into()
        } else {
            parts.join("\n")
        };

        let stats = Paragraph::new(Text::from(content))
            .block(block_bordered(" Tasks "))
            .style(Style::default());
        f.render_widget(stats, area);
    }
}
