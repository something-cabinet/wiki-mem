// ─── Ratatui Terminal UI ─────────────────────────────────────

use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use ratatui::widgets::*;
use ratatui::Frame;

use petgraph::visit::EdgeRef;
use wm_core::engine::VppEngine;
use wm_core::page::get_page;

/// Paste text from the system clipboard (Windows via PowerShell).
/// Returns `None` if clipboard is empty or read fails.
fn paste_from_clipboard() -> Option<String> {
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", "Get-Clipboard"])
        .output()
        .ok()?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if s.is_empty() { None } else { Some(s) }
    } else {
        None
    }
}

/// Run the TUI event loop. Takes ownership of the engine.
pub fn run_tui(engine: Arc<VppEngine>) -> Result<(), anyhow::Error> {
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
                // Help overlay takes priority
                if app.show_help {
                    match key.code {
                        KeyCode::Char('?') | KeyCode::Esc => {
                            app.show_help = false;
                        }
                        _ => {}
                    }
                    continue;
                }

                // Preview overlay takes priority next
                if app.preview_content.is_some() {
                    match key.code {
                        KeyCode::Esc => {
                            app.preview_content = None;
                            app.preview_id = None;
                        }
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => return Ok(()),
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
                            Tab::Dashboard => Tab::Help,
                            Tab::Tasks => Tab::Help, // Tasks not in cycle, wraps to Help
                        };
                    }
                    KeyCode::BackTab => {
                        app.active_tab = match app.active_tab {
                            Tab::Help => Tab::Dashboard,
                            Tab::Dashboard => Tab::Graph,
                            Tab::Graph => Tab::Search,
                            Tab::Search => Tab::Help,
                            Tab::Tasks => Tab::Help,
                        };
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        match app.active_tab {
                            Tab::Dashboard => {
                                if app.dashboard_scroll > 0 {
                                    app.dashboard_scroll -= 1;
                                }
                            }
                            Tab::Search if app.input_mode == InputMode::Results => {
                                if app.list_index > 0 {
                                    app.list_index -= 1;
                                }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        match app.active_tab {
                            Tab::Dashboard => {
                                let snapshot = app.engine.state.graph.load();
                                let total = snapshot.0.node_count();
                                if app.dashboard_scroll + 1 < total {
                                    app.dashboard_scroll += 1;
                                }
                            }
                            Tab::Search if app.input_mode == InputMode::Results => {
                                app.list_index = app
                                    .list_index
                                    .saturating_add(1)
                                    .min(app.search_results.len().saturating_sub(1));
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('i') => {
                        if app.active_tab == Tab::Search {
                            app.input_mode = InputMode::Query;
                        }
                    }
                    KeyCode::Char('\x16') => {
                        // Ctrl+V paste — reads from system clipboard.
                        // NOTE: On some terminals (e.g. Windows Terminal) raw mode may
                        // intercept Ctrl+V before it reaches the app. In that case paste
                        // won't work here; the terminal's own paste handling applies.
                        if app.active_tab == Tab::Search && app.input_mode == InputMode::Query {
                            if let Some(text) = paste_from_clipboard() {
                                app.search_query.push_str(&text);
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        if app.active_tab == Tab::Search && app.input_mode == InputMode::Query {
                            app.search_query.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if app.active_tab == Tab::Search && app.input_mode == InputMode::Query {
                            app.search_query.pop();
                        }
                    }
                    KeyCode::Enter => {
                        if app.active_tab == Tab::Search {
                            if app.input_mode == InputMode::Results
                                && !app.search_results.is_empty()
                            {
                                // Preview the selected result
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
                    }
                    _ => {}
                }
            }
        }
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
    engine: Arc<VppEngine>,
    active_tab: Tab,
    input_mode: InputMode,
    list_index: usize,
    status: String,
    search_query: String,
    search_results: Vec<SearchResult>,
    // Dashboard scroll
    dashboard_scroll: usize,
    // Search preview
    preview_content: Option<String>,
    preview_id: Option<String>,
    // Help overlay
    show_help: bool,
    // Graph
    graph_center: Option<String>,
}

impl App {
    fn new(engine: Arc<VppEngine>) -> Self {
        Self {
            engine,
            active_tab: Tab::Dashboard,
            input_mode: InputMode::Query,
            list_index: 0,
            status: "h/d/s/g/t: tab  Tab/Shift+Tab: cycle  ?: help  q: quit".into(),
            search_query: String::new(),
            search_results: Vec::new(),
            dashboard_scroll: 0,
            preview_content: None,
            preview_id: None,
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

        // Tab bar
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
            .block(Block::bordered().title(" Wiki Memory Engine "))
            .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        f.render_widget(tabs, layout[0]);

        match self.active_tab {
            Tab::Dashboard => self.render_dashboard(f, layout[1]),
            Tab::Search => self.render_search(f, layout[1]),
            Tab::Graph => self.render_graph(f, layout[1]),
            Tab::Tasks => self.render_tasks(f, layout[1]),
            Tab::Help => self.render_help_tab(f, layout[1]),
        }

        // Status bar
        let mut status_text = self.status.clone();
        if self.active_tab == Tab::Dashboard {
            let snapshot = self.engine.state.graph.load();
            let total = snapshot.0.node_count();
            let pos = self.dashboard_scroll + 1;
            if total > 50 {
                status_text.push_str(&format!("  [{}/{}]", pos.min(total), total));
            }
        }
        if self.show_help {
            status_text = "Press ? or Esc to close help overlay".to_string();
        } else if self.active_tab == Tab::Help {
            status_text = "Help tab — ? for overlay, h/tab to navigate away".to_string();
        }
        if self.preview_content.is_some() {
            status_text =
                format!("Preview: {} — Esc to close", self.preview_id.as_deref().unwrap_or(""));
        }
        let status = Paragraph::new(Text::from(status_text.as_str()))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(status, layout[2]);

        // Help overlay on top of everything
        if self.show_help {
            self.render_help(f);
        }
    }

    fn render_help(&self, f: &mut Frame) {
        let area = f.size();
        let help_area = Rect {
            x: area.width / 6,
            y: area.height / 4,
            width: area.width * 2 / 3,
            height: area.height / 2,
        };
        let bindings = vec![
            "q                    Quit",
            "h / d / s / g / t   Switch to Help / Dashboard / Search / Graph / Tasks",
            "Tab                  Cycle tab forward  (help\u{2192}search\u{2192}graph\u{2192}dashboard)",
            "Shift+Tab            Cycle tab backward  (dashboard\u{2192}graph\u{2192}search\u{2192}help)",
            "\u{2191}/k  \u{2193}/j              Navigate list / scroll",
            "Enter                Search / preview result",
            "i (Search tab)       Focus query input",
            "Ctrl+V               Paste from clipboard (Search query)",
            "?                    Toggle this help overlay",
        ];
        let content = Paragraph::new(Text::from(bindings.join("\n")))
            .block(Block::bordered().title(" Help "))
            .style(Style::default())
            .alignment(Alignment::Left);
        f.render_widget(Clear, help_area);
        f.render_widget(content, help_area);
    }

    /// Full-tab view of help content, rendered inside the main content area.
    fn render_help_tab(&self, f: &mut Frame, area: Rect) {
        let bindings = vec![
            ("General", vec![
                "q       Quit",
                "?       Toggle help overlay (on top of current tab)",
            ]),
            ("Navigation", vec![
                "Tab     Cycle tab forward",
                "Shift+Tab  Cycle tab backward",
                "h/d/s/g/t  Switch to Help / Dashboard / Search / Graph / Tasks",
            ]),
            ("Dashboard", vec![
                "\u{2191}/k  \u{2193}/j  Scroll page list",
            ]),
            ("Search", vec![
                "i       Focus query input",
                "Enter   Run search / preview result",
                "Ctrl+V  Paste from clipboard",
                "\u{2191}/k  \u{2193}/j  Navigate results",
                "Esc     Close preview",
            ]),
            ("Graph", vec![
                "(read-only view of centered node and neighbors)",
            ]),
        ];
        let mut content = String::new();
        for (section, keys) in &bindings {
            content.push_str(&format!("{}\n", section));
            for k in keys {
                content.push_str(&format!("  {}\n", k));
            }
            content.push('\n');
        }
        let widget = Paragraph::new(Text::from(content))
            .block(Block::bordered().title(" Help — Key Bindings "))
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
            *page_types.entry(type_name).or_insert(0) += 1;
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
            // Scrollable list with scrollbar (ratatui::widgets::List + Scrollbar)
            let inner = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(area);

            let max_scroll = node_count.saturating_sub(1);
            if self.dashboard_scroll > max_scroll {
                self.dashboard_scroll = max_scroll;
            }

            // Build list items: header info followed by all pages
            let mut items: Vec<ListItem> = vec![
                ListItem::new(Line::from(format!(
                    "Nodes: {}  Edges: {}  Sections: {}  BM25: {}",
                    node_count, edge_count, sections, bm25_docs
                ))),
            ];
            for (t, c) in &page_types {
                items.push(ListItem::new(Line::from(format!("  {}: {}", t, c))));
            }
            items.push(ListItem::new(Line::from("")));
            items.push(ListItem::new(Line::from("Pages:")));
            for idx in graph.node_indices() {
                let meta = &graph[idx];
                items.push(ListItem::new(Line::from(format!("  {} [{}]", meta.title, meta.id))));
            }

            let total_items = items.len();

            // Virtual window of visible items based on scroll position
            let window_size = (inner[0].height as usize).saturating_sub(2); // border
            let window_size = window_size.max(1);
            let start = self.dashboard_scroll.min(total_items.saturating_sub(1));
            let end = (start + window_size).min(total_items);

            let visible_items: Vec<ListItem> = items
                .into_iter()
                .skip(start)
                .take(end - start)
                .collect();

            // Render using List widget with virtual scrolling
            let list = List::new(visible_items)
                .block(Block::bordered().title(" Dashboard "))
                .style(Style::default());
            f.render_widget(list, inner[0]);

            // Scrollbar tracks the scroll offset
            let mut scroll_state =
                ScrollbarState::new(total_items).position(self.dashboard_scroll);
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
                .block(Block::bordered().title(" Dashboard "))
                .style(Style::default());
            f.render_widget(stats, area);
        }
    }

    fn render_search(&mut self, f: &mut Frame, area: Rect) {
        // If preview is active, show preview content
        if let Some(ref content) = self.preview_content {
            let lines: Vec<Line> = content.lines().map(|l| Line::from(l.to_string())).collect();
            let max_lines = (area.height as usize).saturating_sub(2);
            let truncated: Vec<Line> = lines.into_iter().take(max_lines).collect();
            let preview_title = format!(
                " Preview: {} ",
                self.preview_id.as_deref().unwrap_or("")
            );
            let preview = Paragraph::new(Text::from(truncated))
                .block(Block::bordered().title(preview_title))
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(preview, area);
            return;
        }

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        // Query input
        let query_style = if self.input_mode == InputMode::Query {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };
        let input = Paragraph::new(Text::from(format!("> {}", self.search_query)))
            .block(Block::bordered().title(" Query "))
            .style(query_style);
        f.render_widget(input, layout[0]);

        // Results list
        let results_style = if self.input_mode == InputMode::Results {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };
        let result_lines: Vec<Line> = self
            .search_results
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let prefix = if i == self.list_index { " \u{2192} " } else { "    " };
                let pct = (r.score * 100.0) as u8;
                Line::from(format!("{}{:>3}%  {}  {}", prefix, pct, r.id, r.snippet))
            })
            .collect();

        let list = if result_lines.is_empty() {
            Paragraph::new(Text::from("Type a query and press Enter to search."))
                .block(Block::bordered().title(" Results "))
        } else {
            Paragraph::new(Text::from(result_lines))
                .block(Block::bordered().title(format!(
                    " Results ({}) ",
                    self.search_results.len()
                )))
                .style(results_style)
        };
        f.render_widget(list, layout[1]);
    }

    fn render_graph(&self, f: &mut Frame, area: Rect) {
        let snapshot = self.engine.state.graph.load();
        let graph = &snapshot.0;

        let center_id = self.graph_center.as_deref();
        let content = if graph.node_count() == 0 {
            "No pages in wiki.".to_string()
        } else {
            // Find the center node by ID or fall back to first node
            let center_idx: Option<petgraph::graph::NodeIndex> = center_id
                .and_then(|id| snapshot.1.get(id).copied())
                .or_else(|| graph.node_indices().next());

            match center_idx {
                Some(start) => {
                    let meta = &graph[start];
                    let mut lines = format!(
                        "Center: {} [{}]\n\nNeighbors:\n",
                        meta.title, meta.id
                    );
                    for edge in graph.edges(start) {
                        let target = edge.target();
                        let t = &graph[target];
                        let e = format!("{:?}", edge.weight()).to_lowercase();
                        lines.push_str(&format!("  \u{2500}\u{2500}{}\u{2500}\u{2500}\u{25b6} {} [{}]\n", e, t.title, t.id));
                    }
                    if graph.edges(start).count() == 0 {
                        lines.push_str("  (no edges)\n");
                    }
                    // Incoming edges
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
                                "  {} [{}] \u{2500}\u{2500}{}\u{2500}\u{2500}\u{25b6}\n",
                                s.title, s.id, e
                            ));
                        }
                    }
                    lines
                }
                None => "No pages in wiki.".to_string(),
            }
        };

        let title = match center_id {
            Some(id) => format!(" Graph \u{2014} {} ", id),
            None => " Graph ".to_string(),
        };
        let stats = Paragraph::new(Text::from(content))
            .block(Block::bordered().title(title))
            .style(Style::default());
        f.render_widget(stats, area);
    }

    fn render_tasks(&self, f: &mut Frame, area: Rect) {
        let snapshot = self.engine.state.graph.load();
        let graph = &snapshot.0;

        let mut todo = Vec::new();
        let mut in_progress = Vec::new();
        let mut done = Vec::new();

        for idx in graph.node_indices() {
            let meta = &graph[idx];
            if meta.page_type != wm_core::engine::PageType::Task {
                continue;
            }
            match meta.status {
                wm_core::engine::PageStatus::Done => done.push(meta.title.as_str()),
                wm_core::engine::PageStatus::InProgress => in_progress.push(meta.title.as_str()),
                _ => todo.push(meta.title.as_str()),
            }
        }

        let content = format!(
            "TODO ({}):\n{}\n\nIN PROGRESS ({}):\n{}\n\nDONE ({}):\n{}",
            todo.len(),
            todo.iter()
                .map(|t| format!("  \u{25a1} {}\n", t))
                .collect::<String>(),
            in_progress.len(),
            in_progress
                .iter()
                .map(|t| format!("  \u{25d0} {}\n", t))
                .collect::<String>(),
            done.len(),
            done.iter()
                .map(|t| format!("  \u{2713} {}\n", t))
                .collect::<String>(),
        );

        let stats = Paragraph::new(Text::from(content))
            .block(Block::bordered().title(" Tasks "))
            .style(Style::default());
        f.render_widget(stats, area);
    }
}
