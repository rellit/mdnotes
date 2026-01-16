use std::io::{self, Stdout};
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap};

use crate::cli::{AddArgs, EditArgs};
use crate::commands::{add, complete, edit};
use crate::config::{ensure_setup, save_config, Config, SetupOptions};
use crate::models::{Item, ItemKind, Status};
use crate::storage::load_items;
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

#[derive(Default)]
enum InputMode {
    #[default]
    Normal,
    NewItem {
        kind: ItemKind,
        buffer: String,
    },
}

struct ListPane {
    kind: ItemKind,
    items: Vec<Item>,
    state: ListState,
}

impl ListPane {
    fn new(kind: ItemKind, items: Vec<Item>) -> Self {
        let mut pane = Self {
            kind,
            items,
            state: ListState::default(),
        };
        if !pane.items.is_empty() {
            pane.state.select(Some(0));
        }
        pane
    }

    fn set_items(&mut self, items: Vec<Item>) {
        self.items = items;
        if self.items.is_empty() {
            self.state.select(None);
        } else {
            let selected = self.state.selected().unwrap_or(0).min(self.items.len() - 1);
            self.state.select(Some(selected));
        }
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
                    meta.push(format!("prio {}", priority.as_str()));
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
        let list = List::new(items)
            .block(
                Block::default()
                    .title(self.kind.dir_name())
                    .borders(Borders::ALL),
            )
            .highlight_style(highlight)
            .highlight_symbol("▶ ");
        let mut state = self.state.clone();
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
        let notes = load_items(&config, ItemKind::Note)?;
        let tasks = load_items(&config, ItemKind::Task)?;
        Ok(Self {
            tab: ActiveTab::Notes,
            notes: ListPane::new(ItemKind::Note, notes),
            tasks: ListPane::new(ItemKind::Task, tasks),
            settings: SettingsPane::from_config(&config),
            config,
            setup,
            status: String::from("Use arrow keys to navigate, 'n' to add, 'e' to edit, Ctrl+S to save settings, q to quit."),
            input_mode: InputMode::Normal,
            quitting: false,
        })
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> MdResult<()> {
        while !self.quitting {
            terminal
                .draw(|f| self.draw(f))
                .map_err(|e| MdError(e.to_string()))?;
            if event::poll(Duration::from_millis(250)).map_err(|e| MdError(e.to_string()))? {
                if let Event::Key(key) = event::read().map_err(|e| MdError(e.to_string()))? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key)?;
                    }
                }
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
            ])
            .split(frame.size());

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
                    .split(layout[1]);
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

        let status = match &self.input_mode {
            InputMode::NewItem { kind, buffer } => {
                let target = match kind {
                    ItemKind::Note => "note",
                    ItemKind::Task => "task",
                };
                format!("New {target} title: {buffer}")
            }
            InputMode::Normal => self.status.clone(),
        };
        let status_widget = Paragraph::new(status)
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .wrap(Wrap { trim: true });
        frame.render_widget(status_widget, layout[2]);
    }

    fn preview_widget(&self) -> Paragraph<'_> {
        let selected = match self.tab {
            ActiveTab::Notes => self.notes.selected(),
            ActiveTab::Tasks => self.tasks.selected(),
            ActiveTab::Settings => None,
        };
        if let Some(item) = selected {
            let mut lines = Vec::new();
            lines.push(Line::from(format!("# {}", item.title)));
            if let Some(status) = &item.status {
                lines.push(Line::from(format!("status: {}", status.as_str())));
            }
            if let Some(priority) = &item.priority {
                lines.push(Line::from(format!("priority: {}", priority.as_str())));
            }
            if let Some(due) = &item.due {
                lines.push(Line::from(format!("due: {due}")));
            }
            if !item.tags.is_empty() {
                lines.push(Line::from(format!("tags: {}", item.tags.join(", "))));
            }
            lines.push(Line::from("--"));
            for body_line in item.body.lines() {
                lines.push(Line::from(body_line.to_string()));
            }
            Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title("Preview"))
                .wrap(Wrap { trim: false })
        } else {
            Paragraph::new("No item selected")
                .block(Block::default().borders(Borders::ALL).title("Preview"))
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> MdResult<()> {
        if matches!(self.input_mode, InputMode::NewItem { .. }) {
            self.handle_new_item_key(key)
        } else {
            self.handle_normal_key(key)
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
            KeyCode::Char('e') => self.edit_selected()?,
            KeyCode::Char('c') => self.toggle_completion()?,
            KeyCode::Enter => {
                if matches!(self.tab, ActiveTab::Settings) {
                    self.settings.toggle_edit();
                }
            }
            KeyCode::Esc => {
                if matches!(self.tab, ActiveTab::Settings) && self.settings.editing {
                    self.settings.toggle_edit();
                }
            }
            _ => {
                if matches!(self.tab, ActiveTab::Settings) {
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('s')
                    {
                        self.save_settings()?;
                    } else {
                        self.settings.handle_input(&key);
                        match key.code {
                            KeyCode::Up => self.settings.previous_field(),
                            KeyCode::Down => self.settings.next_field(),
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_new_item_key(&mut self, key: KeyEvent) -> MdResult<()> {
        if let InputMode::NewItem { kind, buffer } = &mut self.input_mode {
            match key.code {
                KeyCode::Esc => {
                    self.status = "Cancelled new item".into();
                    self.input_mode = InputMode::Normal;
                }
                KeyCode::Backspace => {
                    buffer.pop();
                }
                KeyCode::Char(c) => buffer.push(c),
                KeyCode::Enter => {
                    let title = buffer.trim();
                    if title.is_empty() {
                        self.status = "Title cannot be empty".into();
                    } else {
                        let kind = kind.clone();
                        let title_owned = title.to_string();
                        self.create_item(kind, title_owned)?;
                        self.input_mode = InputMode::Normal;
                    }
                }
                _ => {}
            }
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
        self.input_mode = InputMode::NewItem {
            kind,
            buffer: String::new(),
        };
    }

    fn create_item(&mut self, kind: ItemKind, title: String) -> MdResult<()> {
        let mut args = AddArgs {
            title: title.clone(),
            body: None,
            due: None,
            status: None,
            priority: None,
            tags: None,
        };
        if matches!(kind, ItemKind::Task) {
            args.status = Some(Status::Pending);
        }
        add::run(args, self.setup.clone())?;
        self.refresh_lists()?;
        self.status = format!("Created {}", title);
        Ok(())
    }

    fn edit_selected(&mut self) -> MdResult<()> {
        let selected = match self.tab {
            ActiveTab::Notes => self.notes.selected(),
            ActiveTab::Tasks => self.tasks.selected(),
            ActiveTab::Settings => None,
        };
        if let Some(item) = selected {
            let id = item.id.clone();
            let title = item.title.clone();
            edit::run(
                EditArgs {
                    id,
                    title: None,
                    body: None,
                    due: None,
                    priority: None,
                    status: None,
                    tags: None,
                },
                self.setup.clone(),
            )?;
            self.refresh_lists()?;
            self.status = format!("Edited {}", title);
        }
        Ok(())
    }

    fn toggle_completion(&mut self) -> MdResult<()> {
        if !matches!(self.tab, ActiveTab::Tasks) {
            return Ok(());
        }
        if let Some(item) = self.tasks.selected() {
            let completed = !matches!(item.status, Some(Status::Completed));
            let id = item.id.clone();
            complete::run(id.clone(), completed, self.setup.clone())?;
            self.refresh_lists()?;
            self.status = format!(
                "Task {} marked {}",
                id,
                if completed { "completed" } else { "pending" }
            );
        }
        Ok(())
    }

    fn refresh_lists(&mut self) -> MdResult<()> {
        self.config = ensure_setup(self.setup.clone())?;
        let notes = load_items(&self.config, ItemKind::Note)?;
        let tasks = load_items(&self.config, ItemKind::Task)?;
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
}
