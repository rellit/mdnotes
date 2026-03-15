use std::io::{self, Stdout};
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap};

use crate::cli::{AddArgs, EditArgs};
use crate::commands::{add, complete, edit};
use crate::config::{Config, SetupOptions, ensure_setup, save_config};
use crate::models::{Item, ItemKind, Status};
use crate::storage::load_all_items;
use crate::{MdError, MdResult};

pub fn run_tui(setup: SetupOptions) -> MdResult<()> {
    let config = ensure_setup(setup.clone())?;
    let mut terminal = init_terminal()?;
    let mut app = App::new(config, setup)?;
    let result = app.run(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}

fn init_terminal() -> MdResult<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode().map_err(|e| MdError(e.to_string()))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| MdError(e.to_string()))?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(|e| MdError(e.to_string()))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> MdResult<()> {
    disable_raw_mode().map_err(|e| MdError(e.to_string()))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(|e| MdError(e.to_string()))?;
    terminal.show_cursor().map_err(|e| MdError(e.to_string()))
}

#[derive(Clone, Copy)]
enum ActiveTab {
    Notes,
    Tasks,
    Settings,
}

impl ActiveTab {
    fn next(self) -> Self {
        match self {
            ActiveTab::Notes => ActiveTab::Tasks,
            ActiveTab::Tasks => ActiveTab::Settings,
            ActiveTab::Settings => ActiveTab::Notes,
        }
    }

    fn prev(self) -> Self {
        match self {
            ActiveTab::Notes => ActiveTab::Settings,
            ActiveTab::Tasks => ActiveTab::Notes,
            ActiveTab::Settings => ActiveTab::Tasks,
        }
    }

    fn index(&self) -> usize {
        match self {
            ActiveTab::Notes => 0,
            ActiveTab::Tasks => 1,
            ActiveTab::Settings => 2,
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1]);
    horizontal[1]
}

fn format_markdown(body: &str) -> Vec<Line<'static>> {
    body.lines().map(highlight_line).collect()
}

fn highlight_line(line: &str) -> Line<'static> {
    if line.trim().is_empty() {
        return Line::from("");
    }
    if line.starts_with('#') {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if line.starts_with("- ") || line.starts_with("* ") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::Yellow),
        ));
    }
    let mut spans: Vec<Span> = Vec::new();
    let mut in_code = false;
    for (idx, part) in line.split('`').enumerate() {
        if idx > 0 {
            in_code = !in_code;
        }
        if part.is_empty() {
            continue;
        }
        let mut style = Style::default();
        if in_code {
            style = style.fg(Color::LightBlue);
        }
        spans.push(Span::styled(part.to_string(), style));
    }
    if spans.is_empty() {
        spans.push(Span::raw(line.to_string()));
    }
    Line::from(spans)
}

#[derive(Clone, Copy)]
enum SettingsField {
    Root,
    Remote,
    Editor,
}

impl SettingsField {
    fn next(self) -> Self {
        match self {
            SettingsField::Root => SettingsField::Remote,
            SettingsField::Remote => SettingsField::Editor,
            SettingsField::Editor => SettingsField::Root,
        }
    }

    fn prev(self) -> Self {
        match self {
            SettingsField::Root => SettingsField::Editor,
            SettingsField::Remote => SettingsField::Root,
            SettingsField::Editor => SettingsField::Remote,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            SettingsField::Root => "Root",
            SettingsField::Remote => "Remote",
            SettingsField::Editor => "Editor",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FormField {
    Title,
    Tags,
    Status,
    Due,
}

impl FormField {
    fn next(self) -> Self {
        match self {
            FormField::Title => FormField::Tags,
            FormField::Tags => FormField::Status,
            FormField::Status => FormField::Due,
            FormField::Due => FormField::Title,
        }
    }

    fn prev(self) -> Self {
        match self {
            FormField::Title => FormField::Due,
            FormField::Tags => FormField::Title,
            FormField::Status => FormField::Tags,
            FormField::Due => FormField::Status,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            FormField::Title => "Title",
            FormField::Tags => "Tags",
            FormField::Status => "Status",
            FormField::Due => "Due",
        }
    }
}

#[derive(Clone)]
enum FormMode {
    New,
    Edit { id: String },
}

#[derive(Clone)]
struct ItemForm {
    mode: FormMode,
    kind: ItemKind,
    title: String,
    tags: String,
    status: Option<Status>,
    due: String,
    active: FormField,
}

impl ItemForm {
    fn new(kind: ItemKind) -> Self {
        Self {
            mode: FormMode::New,
            kind,
            title: String::new(),
            tags: String::new(),
            status: if matches!(kind, ItemKind::Task) {
                Some(Status::Pending)
            } else {
                None
            },
            due: String::new(),
            active: FormField::Title,
        }
    }

    fn from_item(item: &Item) -> Self {
        Self {
            mode: FormMode::Edit {
                id: item.id.clone(),
            },
            kind: item.kind,
            title: item.title.clone(),
            tags: item.tags.join(", "),
            status: item.status,
            due: item.due.clone().unwrap_or_default(),
            active: FormField::Title,
        }
    }

    fn active_label(&self) -> &'static str {
        self.active.label()
    }

    fn next_field(&mut self) {
        self.active = self.active.next();
    }

    fn previous_field(&mut self) {
        self.active = self.active.prev();
    }

    fn cycle_status(&mut self) {
        self.status = match self.status {
            None => Some(Status::Pending),
            Some(Status::Pending) => Some(Status::Completed),
            Some(Status::Completed) => None,
        };
    }

    fn handle_input(&mut self, key: &KeyEvent) {
        if self.active == FormField::Status {
            match key.code {
                KeyCode::Char('p') | KeyCode::Char('P') => self.status = Some(Status::Pending),
                KeyCode::Char('c') | KeyCode::Char('C') => self.status = Some(Status::Completed),
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Backspace => self.status = None,
                KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => self.cycle_status(),
                _ => {}
            }
            return;
        }
        let target = match self.active {
            FormField::Title => Some(&mut self.title),
            FormField::Tags => Some(&mut self.tags),
            FormField::Due => Some(&mut self.due),
            FormField::Status => None,
        };
        if let Some(buffer) = target {
            match key.code {
                KeyCode::Backspace => {
                    buffer.pop();
                }
                KeyCode::Char(c) => buffer.push(c),
                _ => {}
            }
        }
    }

    fn value_for(&self, field: FormField) -> String {
        match field {
            FormField::Title => self.title.clone(),
            FormField::Tags => {
                if self.tags.trim().is_empty() {
                    "<optional>".into()
                } else {
                    self.tags.clone()
                }
            }
            FormField::Status => self
                .status
                .as_ref()
                .map(|s| s.as_str().to_string())
                .unwrap_or_else(|| "<none>".into()),
            FormField::Due => {
                if self.due.trim().is_empty() {
                    "<optional>".into()
                } else {
                    self.due.clone()
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum SortOption {
    Title,
    Due,
    Status,
}

impl SortOption {
    fn next(self) -> Self {
        match self {
            SortOption::Title => SortOption::Due,
            SortOption::Due => SortOption::Status,
            SortOption::Status => SortOption::Title,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            SortOption::Title => "title",
            SortOption::Due => "due",
            SortOption::Status => "status",
        }
    }
}

#[derive(Clone, Default)]
enum InputMode {
    #[default]
    Normal,
    Form(ItemForm),
    Search {
        buffer: String,
    },
}

struct ListPane {
    kind: ItemKind,
    all_items: Vec<Item>,
    items: Vec<Item>,
    state: ListState,
    sort: SortOption,
    status_filter: Option<Status>,
    search_query: Option<String>,
}

impl ListPane {
    fn new(kind: ItemKind, items: Vec<Item>) -> Self {
        let mut pane = Self {
            kind,
            all_items: items.clone(),
            items,
            state: ListState::default(),
            sort: SortOption::Title,
            status_filter: None,
            search_query: None,
        };
        pane.apply_filters();
        pane
    }

    fn set_items(&mut self, items: Vec<Item>) {
        self.all_items = items;
        self.apply_filters();
    }

    fn apply_filters(&mut self) {
        let mut filtered = self.all_items.clone();
        if let Some(filter_status) = &self.status_filter {
            filtered.retain(|item| item.status.as_ref() == Some(filter_status));
        }
        if let Some(query) = &self.search_query {
            let query_lower = query.to_lowercase();
            filtered.retain(|item| {
                item.title.to_lowercase().contains(&query_lower)
                    || item
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
                    || item.body.to_lowercase().contains(&query_lower)
            });
        }
        match self.sort {
            SortOption::Title => {
                filtered.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
            }
            SortOption::Due => {
                filtered.sort_by(|a, b| match (&a.due, &b.due) {
                    (Some(ad), Some(bd)) => ad.cmp(bd),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
                });
            }
            SortOption::Status => {
                filtered.sort_by(|a, b| {
                    let a_status = a.status.as_ref().map(|s| s.as_str()).unwrap_or("");
                    let b_status = b.status.as_ref().map(|s| s.as_str()).unwrap_or("");
                    a_status.cmp(b_status).then(a.title.cmp(&b.title))
                });
            }
        }
        self.items = filtered;
        if self.items.is_empty() {
            self.state.select(None);
        } else {
            let selected = self.state.selected().unwrap_or(0).min(self.items.len() - 1);
            self.state.select(Some(selected));
        }
    }

    fn cycle_sort(&mut self) {
        self.sort = self.sort.next();
        self.apply_filters();
    }

    fn cycle_status_filter(&mut self) {
        if !matches!(self.kind, ItemKind::Task) {
            self.status_filter = None;
            self.apply_filters();
            return;
        }
        self.status_filter = match self.status_filter {
            None => Some(Status::Pending),
            Some(Status::Pending) => Some(Status::Completed),
            Some(Status::Completed) => None,
        };
        self.apply_filters();
    }

    fn set_search_query(&mut self, query: Option<String>) {
        self.search_query = query;
        self.apply_filters();
    }

    fn selected(&self) -> Option<&Item> {
        self.state.selected().and_then(|idx| self.items.get(idx))
    }

    fn select_next(&mut self) {
        if self.items.is_empty() {
            self.state.select(None);
            return;
        }
        let next = match self.state.selected() {
            Some(i) if i + 1 < self.items.len() => i + 1,
            _ => 0,
        };
        self.state.select(Some(next));
    }

    fn select_previous(&mut self) {
        if self.items.is_empty() {
            self.state.select(None);
            return;
        }
        let prev = match self.state.selected() {
            Some(0) | None => self.items.len() - 1,
            Some(i) => i - 1,
        };
        self.state.select(Some(prev));
    }

    fn render(&self, frame: &mut Frame, area: Rect, highlight: Style) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item| {
                let mut meta = Vec::new();
                if let Some(status) = &item.status {
                    meta.push(status.as_str().to_string());
                }
                if let Some(due) = &item.due {
                    meta.push(format!("due {due}"));
                }
                if let Some(priority) = &item.priority {
                    meta.push(format!("prio {priority}"));
                }
                let mut line = item.title.clone();
                if !meta.is_empty() {
                    line.push_str(" (");
                    line.push_str(&meta.join(", "));
                    line.push(')');
                }
                ListItem::new(line)
            })
            .collect();
        let mut title_parts = vec![self.kind.dir_name().to_string()];
        title_parts.push(format!("sort: {}", self.sort.label()));
        if let Some(filter) = &self.status_filter {
            title_parts.push(format!("status: {}", filter.as_str()));
        }
        if let Some(query) = &self.search_query
            && !query.is_empty()
        {
            title_parts.push(format!("search: {query}"));
        }
        let list = List::new(items)
            .block(
                Block::default()
                    .title(title_parts.join(" | "))
                    .borders(Borders::ALL),
            )
            .highlight_style(highlight)
            .highlight_symbol("▶ ");
        let mut state = self.state;
        frame.render_stateful_widget(list, area, &mut state);
    }
}

struct SettingsPane {
    root: String,
    remote: String,
    editor: String,
    active: SettingsField,
    editing: bool,
}

impl SettingsPane {
    fn from_config(config: &Config) -> Self {
        Self {
            root: config.root.display().to_string(),
            remote: config.remote.clone().unwrap_or_default(),
            editor: config.editor.clone().unwrap_or_default(),
            active: SettingsField::Root,
            editing: false,
        }
    }

    fn to_config(&self, base: &Config) -> MdResult<Config> {
        let root_str = self.root.trim();
        if root_str.is_empty() {
            return Err(MdError("Root directory cannot be empty".into()));
        }
        let mut config = base.clone();
        config.root = PathBuf::from(root_str);
        config.remote = if self.remote.trim().is_empty() {
            None
        } else {
            Some(self.remote.trim().to_string())
        };
        config.editor = if self.editor.trim().is_empty() {
            None
        } else {
            Some(self.editor.trim().to_string())
        };
        Ok(config)
    }

    fn next_field(&mut self) {
        self.active = self.active.next();
    }

    fn previous_field(&mut self) {
        self.active = self.active.prev();
    }

    fn toggle_edit(&mut self) {
        self.editing = !self.editing;
    }

    fn handle_input(&mut self, key: &KeyEvent) {
        if !self.editing {
            return;
        }
        match key.code {
            KeyCode::Backspace => {
                let value = self.current_value_mut();
                value.pop();
            }
            KeyCode::Char(c) => {
                let value = self.current_value_mut();
                value.push(c);
            }
            _ => {}
        }
    }

    fn current_value_mut(&mut self) -> &mut String {
        match self.active {
            SettingsField::Root => &mut self.root,
            SettingsField::Remote => &mut self.remote,
            SettingsField::Editor => &mut self.editor,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, highlight: Style) {
        let items: Vec<ListItem> = [
            (SettingsField::Root, &self.root),
            (SettingsField::Remote, &self.remote),
            (SettingsField::Editor, &self.editor),
        ]
        .iter()
        .map(|(field, value)| {
            let mut label = format!("{}: ", field.label());
            if value.is_empty() {
                label.push_str("<unset>");
            } else {
                label.push_str(value);
            }
            ListItem::new(label)
        })
        .collect();

        let mut list = ListState::default();
        list.select(Some(self.active.index()));

        let list_widget = List::new(items)
            .block(Block::default().title("Settings").borders(Borders::ALL))
            .highlight_style(highlight)
            .highlight_symbol("● ");
        frame.render_stateful_widget(list_widget, area, &mut list);
    }
}

impl SettingsField {
    fn index(&self) -> usize {
        match self {
            SettingsField::Root => 0,
            SettingsField::Remote => 1,
            SettingsField::Editor => 2,
        }
    }
}

struct App {
    tab: ActiveTab,
    notes: ListPane,
    tasks: ListPane,
    settings: SettingsPane,
    config: Config,
    setup: SetupOptions,
    status: String,
    input_mode: InputMode,
    quitting: bool,
}

impl App {
    fn new(config: Config, setup: SetupOptions) -> MdResult<Self> {
        let all = load_all_items(&config)?;
        let notes: Vec<Item> = all.iter().filter(|i| !i.is_task()).cloned().collect();
        let tasks: Vec<Item> = all.into_iter().filter(|i| i.is_task()).collect();
        Ok(Self {
            tab: ActiveTab::Notes,
            notes: ListPane::new(ItemKind::Note, notes),
            tasks: ListPane::new(ItemKind::Task, tasks),
            settings: SettingsPane::from_config(&config),
            config,
            setup,
            status: String::from(
                "Use arrows to navigate. n add, e edit, c complete, / search, f filter, o sort, Enter to save edits, q to quit.",
            ),
            input_mode: InputMode::Normal,
            quitting: false,
        })
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> MdResult<()> {
        while !self.quitting {
            terminal
                .draw(|f| self.draw(f))
                .map_err(|e| MdError(e.to_string()))?;
            if event::poll(Duration::from_millis(250)).map_err(|e| MdError(e.to_string()))?
                && let Event::Key(key) = event::read().map_err(|e| MdError(e.to_string()))?
                && key.kind == KeyEventKind::Press
            {
                self.handle_key(key)?;
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(frame.area());
        let main_area = layout[1];

        let titles = ["Notes", "Tasks", "Settings"]
            .iter()
            .map(|t| Line::from(*t))
            .collect::<Vec<_>>();
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("mdnui"))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .select(self.tab.index());
        frame.render_widget(tabs, layout[0]);

        match self.tab {
            ActiveTab::Notes | ActiveTab::Tasks => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
                    .split(main_area);
                let highlight = Style::default().bg(Color::Blue).fg(Color::White);
                match self.tab {
                    ActiveTab::Notes => self.notes.render(frame, chunks[0], highlight),
                    ActiveTab::Tasks => self.tasks.render(frame, chunks[0], highlight),
                    _ => {}
                }
                let preview = self.preview_widget();
                frame.render_widget(preview, chunks[1]);
            }
            ActiveTab::Settings => {
                let highlight = if self.settings.editing {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Cyan)
                };
                self.settings.render(frame, layout[1], highlight);
            }
        }

        let status = self.status.clone();
        let status_widget = Paragraph::new(status)
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .wrap(Wrap { trim: true });
        frame.render_widget(status_widget, layout[2]);

        let commands_widget = self.commands_widget();
        frame.render_widget(commands_widget, layout[3]);

        match &self.input_mode {
            InputMode::Form(form) => {
                let area = centered_rect(70, 60, main_area);
                frame.render_widget(Clear, area);
                let form_widget = self.form_widget(form);
                frame.render_widget(form_widget, area);
            }
            InputMode::Search { buffer } => {
                let area = centered_rect(70, 30, main_area);
                frame.render_widget(Clear, area);
                let search_widget = self.search_widget(buffer);
                frame.render_widget(search_widget, area);
            }
            InputMode::Normal => {}
        }
    }

    fn preview_widget(&self) -> Paragraph<'_> {
        let selected = match self.tab {
            ActiveTab::Notes => self.notes.selected(),
            ActiveTab::Tasks => self.tasks.selected(),
            ActiveTab::Settings => None,
        };
        if let Some(item) = selected {
            let mut lines: Vec<Line> = Vec::new();
            lines.push(Line::from(Span::styled(
                format!("# {}", item.title),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            if let Some(status) = &item.status {
                lines.push(Line::from(vec![
                    Span::styled("status: ", Style::default().fg(Color::Gray)),
                    Span::styled(status.as_str(), Style::default().fg(Color::Yellow)),
                ]));
            }
            if let Some(priority) = &item.priority {
                lines.push(Line::from(vec![
                    Span::styled("priority: ", Style::default().fg(Color::Gray)),
                    Span::styled(priority.to_string(), Style::default().fg(Color::Green)),
                ]));
            }
            if let Some(due) = &item.due {
                lines.push(Line::from(vec![
                    Span::styled("due: ", Style::default().fg(Color::Gray)),
                    Span::styled(due, Style::default().fg(Color::Magenta)),
                ]));
            }
            if !item.tags.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("tags: ", Style::default().fg(Color::Gray)),
                    Span::styled(item.tags.join(", "), Style::default().fg(Color::LightBlue)),
                ]));
            }
            lines.push(Line::from("--"));
            lines.extend(format_markdown(&item.body));
            Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title("Preview"))
                .wrap(Wrap { trim: false })
        } else {
            Paragraph::new("No item selected")
                .block(Block::default().borders(Borders::ALL).title("Preview"))
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> MdResult<()> {
        match &self.input_mode {
            InputMode::Form(_) => self.handle_form_key(key),
            InputMode::Search { .. } => self.handle_search_key(key),
            InputMode::Normal => self.handle_normal_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> MdResult<()> {
        match key.code {
            KeyCode::Char('q') => {
                self.quitting = true;
            }
            KeyCode::Left => self.tab = self.tab.prev(),
            KeyCode::Right => self.tab = self.tab.next(),
            KeyCode::Up => self.move_selection_up(),
            KeyCode::Down => self.move_selection_down(),
            KeyCode::Char('r') => self.refresh_lists()?,
            KeyCode::Char('n') => self.start_new_item(),
            KeyCode::Char('e') => self.start_edit_form(),
            KeyCode::Char('c') => self.toggle_completion()?,
            KeyCode::Char('f') => {
                if !matches!(self.tab, ActiveTab::Settings) {
                    self.toggle_filter();
                }
            }
            KeyCode::Char('o') => {
                if !matches!(self.tab, ActiveTab::Settings) {
                    self.toggle_sort();
                }
            }
            KeyCode::Char('/') => self.start_search(),
            KeyCode::Enter => {
                if matches!(self.tab, ActiveTab::Settings) {
                    if self.settings.editing {
                        self.save_settings()?;
                        self.settings.toggle_edit();
                    } else {
                        self.settings.toggle_edit();
                    }
                }
            }
            KeyCode::Esc => {
                if matches!(self.tab, ActiveTab::Settings) && self.settings.editing {
                    self.settings.toggle_edit();
                }
            }
            _ => {
                if matches!(self.tab, ActiveTab::Settings) {
                    self.settings.handle_input(&key);
                    match key.code {
                        KeyCode::Up => self.settings.previous_field(),
                        KeyCode::Down => self.settings.next_field(),
                        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.save_settings()?;
                            if self.settings.editing {
                                self.settings.toggle_edit();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_form_key(&mut self, key: KeyEvent) -> MdResult<()> {
        if let InputMode::Form(mut form) = self.input_mode.clone() {
            match key.code {
                KeyCode::Esc => {
                    self.status = "Cancelled input".into();
                    self.input_mode = InputMode::Normal;
                    return Ok(());
                }
                KeyCode::Tab => form.next_field(),
                KeyCode::BackTab => form.previous_field(),
                KeyCode::Enter => {
                    self.submit_form(form)?;
                    return Ok(());
                }
                _ => form.handle_input(&key),
            }
            self.input_mode = InputMode::Form(form);
        }
        Ok(())
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> MdResult<()> {
        if let InputMode::Search { mut buffer } = self.input_mode.clone() {
            match key.code {
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    return Ok(());
                }
                KeyCode::Backspace => {
                    buffer.pop();
                }
                KeyCode::Char(c) => buffer.push(c),
                KeyCode::Enter => {
                    self.apply_search(buffer.trim());
                    return Ok(());
                }
                _ => {}
            }
            self.input_mode = InputMode::Search { buffer };
        }
        Ok(())
    }

    fn move_selection_up(&mut self) {
        match self.tab {
            ActiveTab::Notes => self.notes.select_previous(),
            ActiveTab::Tasks => self.tasks.select_previous(),
            ActiveTab::Settings => self.settings.previous_field(),
        }
    }

    fn move_selection_down(&mut self) {
        match self.tab {
            ActiveTab::Notes => self.notes.select_next(),
            ActiveTab::Tasks => self.tasks.select_next(),
            ActiveTab::Settings => self.settings.next_field(),
        }
    }

    fn start_new_item(&mut self) {
        let kind = match self.tab {
            ActiveTab::Notes => ItemKind::Note,
            ActiveTab::Tasks => ItemKind::Task,
            ActiveTab::Settings => {
                self.settings.toggle_edit();
                return;
            }
        };
        self.input_mode = InputMode::Form(ItemForm::new(kind));
        self.status = "Adding new item - press Enter to save".into();
    }

    fn start_edit_form(&mut self) {
        let selected = match self.tab {
            ActiveTab::Notes => self.notes.selected(),
            ActiveTab::Tasks => self.tasks.selected(),
            ActiveTab::Settings => None,
        };
        if let Some(item) = selected {
            self.input_mode = InputMode::Form(ItemForm::from_item(item));
            self.status = "Editing item - press Enter to save".into();
        }
    }

    fn toggle_completion(&mut self) -> MdResult<()> {
        if !matches!(self.tab, ActiveTab::Tasks) {
            return Ok(());
        }
        if let Some(item) = self.tasks.selected() {
            let completed = !matches!(item.status, Some(Status::Completed));
            let id = item.id.clone();
            complete::run(id.clone(), completed, self.setup.clone(), false)?;
            self.refresh_lists()?;
            self.status = format!(
                "Task {} marked {}",
                id,
                if completed { "completed" } else { "pending" }
            );
        }
        Ok(())
    }

    fn toggle_filter(&mut self) {
        match self.tab {
            ActiveTab::Notes => self.notes.cycle_status_filter(),
            ActiveTab::Tasks => self.tasks.cycle_status_filter(),
            ActiveTab::Settings => return,
        };
        let status_text = match self.tab {
            ActiveTab::Notes => self.notes.status_filter.as_ref().map(|s| s.as_str()),
            ActiveTab::Tasks => self.tasks.status_filter.as_ref().map(|s| s.as_str()),
            ActiveTab::Settings => None,
        };
        self.status = match status_text {
            Some(s) => format!("Filter: {s}"),
            None => "Filter cleared".into(),
        };
    }

    fn toggle_sort(&mut self) {
        match self.tab {
            ActiveTab::Notes => self.notes.cycle_sort(),
            ActiveTab::Tasks => self.tasks.cycle_sort(),
            ActiveTab::Settings => {}
        }
        let label = match self.tab {
            ActiveTab::Notes => self.notes.sort.label(),
            ActiveTab::Tasks => self.tasks.sort.label(),
            ActiveTab::Settings => "n/a",
        };
        self.status = format!("Sort: {label}");
    }

    fn start_search(&mut self) {
        let buffer = match self.tab {
            ActiveTab::Notes => self.notes.search_query.clone().unwrap_or_default(),
            ActiveTab::Tasks => self.tasks.search_query.clone().unwrap_or_default(),
            ActiveTab::Settings => String::new(),
        };
        if matches!(self.tab, ActiveTab::Settings) {
            return;
        }
        self.input_mode = InputMode::Search { buffer };
        self.status = "Search current list - Enter to apply".into();
    }

    fn apply_search(&mut self, query: &str) {
        let trimmed = query.trim();
        let query_opt = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
        match self.tab {
            ActiveTab::Notes => self.notes.set_search_query(query_opt),
            ActiveTab::Tasks => self.tasks.set_search_query(query_opt),
            ActiveTab::Settings => {}
        }
        if trimmed.is_empty() {
            self.status = "Search cleared".into();
        } else {
            self.status = format!("Searching for '{trimmed}'");
        }
        self.input_mode = InputMode::Normal;
    }

    fn submit_form(&mut self, form: ItemForm) -> MdResult<()> {
        let title = form.title.trim();
        if title.is_empty() {
            self.status = "Title cannot be empty".into();
            self.input_mode = InputMode::Form(form);
            return Ok(());
        }
        let tags_value = form.tags.trim().to_string();
        let clear_due = form.due.trim().is_empty();
        let due_value = if clear_due {
            None
        } else {
            match crate::util::validate_due_inner(form.due.trim()) {
                Ok(val) => Some(val),
                Err(err) => {
                    self.status = err.0;
                    self.input_mode = InputMode::Form(form);
                    return Ok(());
                }
            }
        };
        match &form.mode {
            FormMode::New => {
                let mut args = AddArgs {
                    title: title.to_string(),
                    body: None,
                    due: due_value.clone(),
                    status: form.status,
                    priority: None,
                    tags: if tags_value.is_empty() {
                        None
                    } else {
                        Some(tags_value.clone())
                    },
                };
                if matches!(form.kind, ItemKind::Task) && args.status.is_none() {
                    args.status = Some(Status::Pending);
                }
                add::run(args, self.setup.clone())?;
                self.status = format!("Created {}", title);
            }
            FormMode::Edit { id } => {
                let args = EditArgs {
                    id: id.to_string(),
                    title: Some(title.to_string()),
                    body: None,
                    due: if clear_due {
                        Some(String::new())
                    } else {
                        due_value.clone()
                    },
                    priority: None,
                    status: form.status,
                    tags: Some(tags_value.clone()),
                };
                edit::run(args, self.setup.clone())?;
                self.status = format!("Updated {}", title);
            }
        }
        self.refresh_lists()?;
        self.input_mode = InputMode::Normal;
        Ok(())
    }

    fn refresh_lists(&mut self) -> MdResult<()> {
        self.config = ensure_setup(self.setup.clone())?;
        let all = load_all_items(&self.config)?;
        let notes: Vec<Item> = all.iter().filter(|i| !i.is_task()).cloned().collect();
        let tasks: Vec<Item> = all.into_iter().filter(|i| i.is_task()).collect();
        self.notes.set_items(notes);
        self.tasks.set_items(tasks);
        Ok(())
    }

    fn save_settings(&mut self) -> MdResult<()> {
        let updated = self.settings.to_config(&self.config)?;
        save_config(&self.setup, &updated)?;
        self.setup = SetupOptions {
            root_override: Some(updated.root.clone()),
            config_home: self.setup.config_home.clone(),
            remote_override: updated.remote.clone(),
            editor_override: updated.editor.clone(),
        };
        self.config = ensure_setup(self.setup.clone())?;
        self.settings = SettingsPane::from_config(&self.config);
        self.refresh_lists()?;
        self.status = "Settings saved".into();
        Ok(())
    }

    fn commands_widget(&self) -> Paragraph<'_> {
        let text = match &self.input_mode {
            InputMode::Form(form) => format!(
                "[Enter] save • [Esc] cancel • [Tab/Shift+Tab] move • Editing {}",
                form.active_label()
            ),
            InputMode::Search { .. } => "[Enter] apply search • [Esc] cancel • type to edit query"
                .to_string(),
            InputMode::Normal => match self.tab {
                ActiveTab::Notes | ActiveTab::Tasks => {
                    "↑/↓ move • ←/→ tabs • n new • e edit • c complete (tasks) • o sort • f filter • / search • r refresh • q quit".into()
                }
                ActiveTab::Settings => {
                    "↑/↓ choose field • Enter edit/save • Esc cancel • Ctrl+S save • q quit".into()
                }
            },
        };
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Commands"))
            .wrap(Wrap { trim: true })
    }

    fn form_widget(&self, form: &ItemForm) -> Paragraph<'_> {
        let mut lines = Vec::new();
        let title = match &form.mode {
            FormMode::New => format!(
                "New {}",
                match form.kind {
                    ItemKind::Note => "Note",
                    ItemKind::Task => "Task",
                }
            ),
            FormMode::Edit { .. } => "Edit Item".into(),
        };
        for field in [
            FormField::Title,
            FormField::Tags,
            FormField::Status,
            FormField::Due,
        ] {
            let value = form.value_for(field);
            let content = format!("{}: {}", field.label(), value);
            let mut style = Style::default();
            if field == form.active {
                style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
            }
            lines.push(Line::from(Span::styled(content, style)));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(
            "Status: cycle with arrows or p/c/n • Due format YYYY-MM-DD",
        ));
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title))
    }

    fn search_widget(&self, buffer: &str) -> Paragraph<'_> {
        let mut lines = Vec::new();
        lines.push(Line::from("Search current list (title, body, tags)"));
        lines.push(Line::from(format!("Query: {buffer}")));
        lines.push(Line::from("Enter to apply • empty to clear"));
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Search"))
            .wrap(Wrap { trim: true })
    }
}
