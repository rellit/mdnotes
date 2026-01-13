use std::env;
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

const CONFIG_NAME: &str = "mdnrc";
const DEFAULT_ROOT_DIR: &str = "repository";
const CONFIG_OVERRIDE_ENV: &str = "MDNOTES_CONFIG_HOME";
const ROOT_OVERRIDE_ENV: &str = "MDNOTES_ROOT";

fn main() {
    if let Err(err) = dispatch() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn dispatch() -> Result<(), String> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        print_help();
        return Ok(());
    }

    let command = args.remove(0);
    match command.as_str() {
        "setup" => handle_setup(&args),
        "add" | "a" => handle_add(&args),
        "list" | "ls" | "l" => handle_list(&args),
        "delete" | "del" | "d" => handle_delete(&args),
        "edit" | "e" => handle_edit(&args),
        "search" | "s" => handle_search(&args),
        "complete" | "c" => handle_complete(&args, true),
        "incomplete" | "ic" => handle_complete(&args, false),
        "show" | "sh" => handle_show(&args),
        other => Err(format!("Unknown command '{other}'"))?,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    fn from_str(input: &str) -> Option<Self> {
        match input.to_ascii_lowercase().as_str() {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Status {
    Pending,
    Completed,
}

impl Status {
    fn from_str(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "pending" => Some(Self::Pending),
            "completed" | "complete" => Some(Self::Completed),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Status::Pending => "pending",
            Status::Completed => "completed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ItemKind {
    Note,
    Task,
}

impl ItemKind {
    fn dir_name(&self) -> &'static str {
        match self {
            ItemKind::Note => "notes",
            ItemKind::Task => "tasks",
        }
    }
}

#[derive(Debug, Clone)]
struct Item {
    id: String,
    title: String,
    kind: ItemKind,
    body: String,
    tags: Vec<String>,
    status: Option<Status>,
    priority: Option<Priority>,
    due: Option<String>,
}

#[derive(Debug, Clone)]
struct Config {
    root: PathBuf,
    remote: Option<String>,
}

fn print_help() {
    println!("mdnotes - A simple note-taking and task management app");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Commands:");
    println!("  setup [root]                Initialize config and storage");
    println!("  add|a note|n <title> [options]   Create a new note");
    println!("  add|a task|t <title> [options]   Create a new task");
    println!("  list|ls notes|tasks [filters]    List stored items");
    println!("  edit|e <id/prefix> [options]     Edit an existing item");
    println!("  delete|del|d <id/prefix>         Delete a note/task");
    println!("  search|s <query>                 Search notes by title/content");
    println!("  complete|c <task id>             Mark task complete");
    println!("  incomplete|ic <task id>          Mark task pending");
    println!("  show|sh <id/prefix>              Show full item content");
    println!();
    println!("Tags can be provided with --tags tag1,tag2");
    println!("Priority: --priority low|medium|high, Status: --status pending|completed");
    println!("Due date for tasks: --due YYYY-MM-DD");
}

fn handle_setup(args: &[String]) -> Result<(), String> {
    let root_override = args.get(0).map(PathBuf::from);
    let config = ensure_setup(root_override)?;
    println!("Config: {}", config_path().display());
    println!("Root: {}", config.root.display());
    Ok(())
}

fn handle_add(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Specify 'note' or 'task'".into());
    }
    let kind = match args[0].as_str() {
        "note" | "n" => ItemKind::Note,
        "task" | "t" => ItemKind::Task,
        other => return Err(format!("Unknown item kind '{other}'")),
    };
    let config = ensure_setup(None)?;
    let mut title: Option<String> = None;
    let mut body = String::new();
    let mut tags: Vec<String> = Vec::new();
    let mut priority: Option<Priority> = None;
    let mut due: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--tags" => {
                if let Some(raw) = args.get(i + 1) {
                    tags = parse_tags(raw);
                }
                i += 2;
            }
            "--body" => {
                if let Some(raw) = args.get(i + 1) {
                    body = raw.clone();
                }
                i += 2;
            }
            "--priority" => {
                if let Some(raw) = args.get(i + 1) {
                    priority = Priority::from_str(raw);
                }
                i += 2;
            }
            "--due" => {
                if let Some(raw) = args.get(i + 1) {
                    due = Some(raw.clone());
                }
                i += 2;
            }
            other => {
                if title.is_none() {
                    title = Some(other.to_string());
                }
                i += 1;
            }
        }
    }

    let title = title.ok_or_else(|| "Title is required".to_string())?;
    let mut item = Item {
        id: Uuid::new_v4().to_string(),
        title,
        kind,
        body,
        tags,
        status: None,
        priority,
        due,
    };
    if matches!(item.kind, ItemKind::Task) {
        if item.status.is_none() {
            item.status = Some(Status::Pending);
        }
    }
    let path = write_item(&config, &item)?;
    refresh_tag_links(&config, &item)?;
    println!("Created {}", path.display());
    Ok(())
}

fn handle_list(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Specify 'notes' or 'tasks'".into());
    }
    let target = args[0].as_str();
    let config = ensure_setup(None)?;
    let filter_status = find_option(args, "--status").and_then(|v| Status::from_str(&v));
    let filter_priority = find_option(args, "--priority").and_then(|v| Priority::from_str(&v));
    match target {
        "notes" | "note" | "n" => {
            let notes = load_items(&config, ItemKind::Note)?;
            for note in notes {
                println!("{} - {}{}", note.id, note.title, format_tags(&note.tags));
            }
        }
        "tasks" | "task" | "t" => {
            let tasks = load_items(&config, ItemKind::Task)?;
            for task in tasks {
                if let Some(filter) = &filter_status {
                    if task.status.as_ref() != Some(filter) {
                        continue;
                    }
                }
                if let Some(filter) = &filter_priority {
                    if task.priority.as_ref() != Some(filter) {
                        continue;
                    }
                }
                println!(
                    "{} - {} [{}{}{}]",
                    task.id,
                    task.title,
                    task.status
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("pending"),
                    task.priority
                        .as_ref()
                        .map(|p| format!(", {}", p.as_str()))
                        .unwrap_or_default(),
                    task.due
                        .as_ref()
                        .map(|d| format!(", due {d}"))
                        .unwrap_or_default()
                );
            }
        }
        other => return Err(format!("Unknown list target '{other}'")),
    }
    Ok(())
}

fn handle_delete(args: &[String]) -> Result<(), String> {
    let id = args
        .get(0)
        .ok_or_else(|| "Provide an id or prefix".to_string())?;
    let config = ensure_setup(None)?;
    let (kind, path, _) = resolve_item(&config, id)?;
    fs::remove_file(&path).map_err(|e| e.to_string())?;
    remove_tag_links(&config, &path, &kind)?;
    println!("Deleted {}", path.display());
    Ok(())
}

fn handle_edit(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Provide an id or prefix to edit".into());
    }
    let id = &args[0];
    let config = ensure_setup(None)?;
    let (kind, path, mut item) = resolve_item(&config, id)?;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--title" => {
                if let Some(v) = args.get(i + 1) {
                    item.title = v.clone();
                }
                i += 2;
            }
            "--body" => {
                if let Some(v) = args.get(i + 1) {
                    item.body = v.clone();
                }
                i += 2;
            }
            "--tags" => {
                if let Some(v) = args.get(i + 1) {
                    item.tags = parse_tags(v);
                }
                i += 2;
            }
            "--due" => {
                if let Some(v) = args.get(i + 1) {
                    item.due = Some(v.clone());
                }
                i += 2;
            }
            "--priority" => {
                if let Some(v) = args.get(i + 1) {
                    item.priority = Priority::from_str(v);
                }
                i += 2;
            }
            "--status" => {
                if let Some(v) = args.get(i + 1) {
                    item.status = Status::from_str(v);
                }
                i += 2;
            }
            other => {
                return Err(format!("Unknown option '{other}' for edit"));
            }
        }
    }
    item.kind = kind.clone();
    write_item(&config, &item)?;
    refresh_tag_links(&config, &item)?;
    println!("Updated {}", path.display());
    Ok(())
}

fn handle_search(args: &[String]) -> Result<(), String> {
    let query = args
        .get(0)
        .ok_or_else(|| "Provide a query string".to_string())?;
    let config = ensure_setup(None)?;
    let notes = load_items(&config, ItemKind::Note)?;
    for note in notes {
        if note
            .title
            .to_ascii_lowercase()
            .contains(&query.to_ascii_lowercase())
            || note
                .body
                .to_ascii_lowercase()
                .contains(&query.to_ascii_lowercase())
        {
            println!("{} - {}", note.id, note.title);
        }
    }
    Ok(())
}

fn handle_complete(args: &[String], completed: bool) -> Result<(), String> {
    let id = args
        .get(0)
        .ok_or_else(|| "Provide a task id or prefix".to_string())?;
    let config = ensure_setup(None)?;
    let (kind, _path, mut item) = resolve_item(&config, id)?;
    if !matches!(kind, ItemKind::Task) {
        return Err("Completion can only be toggled for tasks".into());
    }
    item.status = Some(if completed {
        Status::Completed
    } else {
        Status::Pending
    });
    write_item(&config, &item)?;
    refresh_tag_links(&config, &item)?;
    println!(
        "Task {} marked {}",
        item.id,
        item.status.as_ref().unwrap().as_str()
    );
    Ok(())
}

fn handle_show(args: &[String]) -> Result<(), String> {
    let id = args
        .get(0)
        .ok_or_else(|| "Provide an id or prefix to show".to_string())?;
    let config = ensure_setup(None)?;
    let (_kind, path, item) = resolve_item(&config, id)?;
    println!("# {}", item.title);
    if let Some(status) = &item.status {
        println!("status: {}", status.as_str());
    }
    if let Some(priority) = &item.priority {
        println!("priority: {}", priority.as_str());
    }
    if let Some(due) = &item.due {
        println!("due: {due}");
    }
    if !item.tags.is_empty() {
        println!("tags: {}", item.tags.join(", "));
    }
    println!("--");
    println!("{}", item.body);
    println!();
    println!("{}", path.display());
    Ok(())
}

fn find_option(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == name)
        .and_then(|w| w.get(1))
        .cloned()
}

fn parse_tags(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn format_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", tags.join(", "))
    }
}

fn config_home() -> PathBuf {
    if let Some(override_dir) = env::var_os(CONFIG_OVERRIDE_ENV) {
        return PathBuf::from(override_dir);
    }
    if cfg!(target_os = "windows") {
        if let Some(appdata) = env::var_os("APPDATA") {
            return PathBuf::from(appdata).join("mdnotes");
        }
    } else if cfg!(target_os = "macos") {
        if let Some(home) = dirs_home() {
            return home
                .join("Library")
                .join("Application Support")
                .join("mdnotes");
        }
    }
    dirs_home()
        .map(|home| home.join(".config").join("mdnotes"))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn dirs_home() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from).or_else(|| {
        #[cfg(target_os = "windows")]
        {
            env::var_os("USERPROFILE").map(PathBuf::from)
        }
        #[cfg(not(target_os = "windows"))]
        {
            None
        }
    })
}

fn config_path() -> PathBuf {
    config_home().join(CONFIG_NAME)
}

fn ensure_setup(root_override: Option<PathBuf>) -> Result<Config, String> {
    let config_file = config_path();
    if config_file.exists() {
        let mut config = read_config(&config_file)?;
        if let Some(root) = root_override {
            config.root = root;
            write_config(&config_file, &config)?;
        }
        ensure_directories(&config.root)?;
        return Ok(config);
    }

    let root = root_override
        .or_else(|| env::var_os(ROOT_OVERRIDE_ENV).map(PathBuf::from))
        .unwrap_or_else(|| config_home().join(DEFAULT_ROOT_DIR));
    let config = Config { root, remote: None };
    write_config(&config_file, &config)?;
    ensure_directories(&config.root)?;
    initialize_git(&config.root);
    Ok(config)
}

fn write_config(path: &Path, config: &Config) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = File::create(path).map_err(|e| e.to_string())?;
    writeln!(file, "root={}", config.root.display()).map_err(|e| e.to_string())?;
    if let Some(remote) = &config.remote {
        writeln!(file, "remote={remote}").map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn read_config(path: &Path) -> Result<Config, String> {
    let mut content = String::new();
    File::open(path)
        .map_err(|e| e.to_string())?
        .read_to_string(&mut content)
        .map_err(|e| e.to_string())?;

    let mut root: Option<PathBuf> = None;
    let mut remote: Option<String> = None;
    for line in content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "root" => root = Some(PathBuf::from(value.trim())),
                "remote" => remote = Some(value.trim().to_string()),
                _ => {}
            }
        }
    }
    let root = root.ok_or_else(|| "Invalid config: missing root".to_string())?;
    Ok(Config { root, remote })
}

fn ensure_directories(root: &Path) -> Result<(), String> {
    fs::create_dir_all(root.join("notes")).map_err(|e| e.to_string())?;
    fs::create_dir_all(root.join("tasks")).map_err(|e| e.to_string())?;
    fs::create_dir_all(root.join("tags")).map_err(|e| e.to_string())?;
    Ok(())
}

fn initialize_git(root: &Path) {
    if root.join(".git").exists() {
        return;
    }
    let _ = Command::new("git").arg("init").arg(root).output();
}

fn write_item(config: &Config, item: &Item) -> Result<PathBuf, String> {
    ensure_directories(&config.root)?;
    let dir = config.root.join(item.kind.dir_name());
    let path = dir.join(format!("{}.md", item.id));
    let mut file = File::create(&path).map_err(|e| e.to_string())?;
    writeln!(file, "id: {}", item.id).map_err(|e| e.to_string())?;
    writeln!(file, "title: {}", item.title).map_err(|e| e.to_string())?;
    writeln!(
        file,
        "type: {}",
        match item.kind {
            ItemKind::Note => "note",
            ItemKind::Task => "task",
        }
    )
    .map_err(|e| e.to_string())?;
    if let Some(status) = &item.status {
        writeln!(file, "status: {}", status.as_str()).map_err(|e| e.to_string())?;
    }
    if let Some(priority) = &item.priority {
        writeln!(file, "priority: {}", priority.as_str()).map_err(|e| e.to_string())?;
    }
    if let Some(due) = &item.due {
        writeln!(file, "due: {}", due).map_err(|e| e.to_string())?;
    }
    if !item.tags.is_empty() {
        writeln!(file, "tags: {}", item.tags.join(", ")).map_err(|e| e.to_string())?;
    }
    writeln!(file, "--").map_err(|e| e.to_string())?;
    write!(file, "{}", item.body).map_err(|e| e.to_string())?;
    Ok(path)
}

fn load_items(config: &Config, kind: ItemKind) -> Result<Vec<Item>, String> {
    let dir = config.root.join(kind.dir_name());
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            if let Ok(item) = read_item(&path, kind.clone()) {
                out.push(item);
            }
        }
    }
    Ok(out)
}

fn read_item(path: &Path, fallback_kind: ItemKind) -> Result<Item, String> {
    let mut content = String::new();
    File::open(path)
        .map_err(|e| e.to_string())?
        .read_to_string(&mut content)
        .map_err(|e| e.to_string())?;
    let mut id: Option<String> = None;
    let mut title: Option<String> = None;
    let mut tags: Vec<String> = Vec::new();
    let mut status: Option<Status> = None;
    let mut priority: Option<Priority> = None;
    let mut due: Option<String> = None;
    let mut kind = fallback_kind.clone();
    let mut body = String::new();
    let mut in_body = false;
    for line in content.lines() {
        if in_body {
            if !body.is_empty() {
                body.push('\n');
            }
            body.push_str(line);
            continue;
        }
        if line.trim() == "--" {
            in_body = true;
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let v = value.trim();
            match key.trim() {
                "id" => id = Some(v.to_string()),
                "title" => title = Some(v.to_string()),
                "tags" => tags = parse_tags(v),
                "status" => status = Status::from_str(v),
                "priority" => priority = Priority::from_str(v),
                "due" => due = Some(v.to_string()),
                "type" => {
                    kind = match v {
                        "note" => ItemKind::Note,
                        "task" => ItemKind::Task,
                        _ => fallback_kind.clone(),
                    }
                }
                _ => {}
            }
        }
    }
    let id = id
        .or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .ok_or_else(|| "Missing id".to_string())?;
    let title = title.ok_or_else(|| "Missing title".to_string())?;
    Ok(Item {
        id,
        title,
        kind,
        body,
        tags,
        status,
        priority,
        due,
    })
}

fn resolve_item(config: &Config, prefix: &str) -> Result<(ItemKind, PathBuf, Item), String> {
    let mut matches: Vec<(ItemKind, PathBuf)> = Vec::new();
    for kind in [ItemKind::Note, ItemKind::Task] {
        let dir = config.root.join(kind.dir_name());
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem.starts_with(prefix) {
                    matches.push((kind.clone(), path));
                }
            }
        }
    }
    if matches.is_empty() {
        return Err(format!("No item matches prefix '{prefix}'"));
    }
    if matches.len() > 1 {
        let mut msg = String::from("Multiple matches:\n");
        for (k, p) in matches {
            let _ = writeln!(
                msg,
                "- {} ({})",
                p.file_name().and_then(|s| s.to_str()).unwrap_or_default(),
                k.dir_name()
            );
        }
        return Err(msg);
    }
    let (kind, path) = matches.into_iter().next().unwrap();
    let item = read_item(&path, kind.clone())?;
    Ok((kind, path, item))
}

fn refresh_tag_links(config: &Config, item: &Item) -> Result<(), String> {
    let path = config
        .root
        .join(item.kind.dir_name())
        .join(format!("{}.md", item.id));
    remove_tag_links(config, &path, &item.kind)?;
    for tag in &item.tags {
        let dir = config.root.join("tags").join(tag);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let link = dir.join(format!("{}.lnk", item.id));
        if link.exists() {
            let _ = fs::remove_file(&link);
        }
        create_symlink(&path, &link)?;
    }
    Ok(())
}

fn remove_tag_links(config: &Config, path: &Path, kind: &ItemKind) -> Result<(), String> {
    let tags_dir = config.root.join("tags");
    if !tags_dir.exists() {
        return Ok(());
    }
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    for entry in fs::read_dir(&tags_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let tag_dir = entry.path();
        if !tag_dir.is_dir() {
            continue;
        }
        let link_path = tag_dir.join(format!("{}.lnk", stem));
        if link_path.exists() {
            let _ = fs::remove_file(link_path);
        }
    }
    // clean tag directories that became empty
    for entry in fs::read_dir(&tags_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let tag_dir = entry.path();
        if tag_dir.is_dir()
            && fs::read_dir(&tag_dir)
                .map_err(|e| e.to_string())?
                .next()
                .is_none()
        {
            let _ = fs::remove_dir(tag_dir);
        }
    }
    // ensure base dirs exist after cleanup
    let _ = ensure_directories(&config.root);
    // keep notes/tasks directories intact
    let _ = kind;
    Ok(())
}

fn create_symlink(target: &Path, link: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::os::windows::fs::symlink_file(target, link).map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::os::unix::fs::symlink(target, link).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;
    use std::sync::Mutex;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn temp_home(test: &str) -> PathBuf {
        let base = std::env::temp_dir().join("mdnotes-tests").join(test);
        if base.exists() {
            let _ = fs::remove_dir_all(&base);
        }
        fs::create_dir_all(&base).unwrap();
        base
    }

    fn set_test_env(base: &Path) {
        env::set_var(CONFIG_OVERRIDE_ENV, base.join("config"));
        env::set_var(ROOT_OVERRIDE_ENV, base.join("repo"));
    }

    #[test]
    fn setup_creates_directories_and_config() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let base = temp_home("setup");
        set_test_env(&base);
        let config = ensure_setup(None).unwrap();
        assert!(config.root.join("notes").exists());
        assert!(config.root.join("tasks").exists());
        assert!(config.root.join("tags").exists());
        let mut content = String::new();
        File::open(config_path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        assert!(content.contains("root="));
    }

    #[test]
    fn add_and_list_note() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let base = temp_home("add_note");
        set_test_env(&base);
        handle_add(&vec![
            "note".into(),
            "My Note".into(),
            "--body".into(),
            "hello".into(),
            "--tags".into(),
            "rust,notes".into(),
        ])
        .unwrap();
        let config = ensure_setup(None).unwrap();
        let notes = load_items(&config, ItemKind::Note).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title, "My Note");
        assert!(notes[0].tags.contains(&"rust".into()));
        assert_eq!(notes[0].body, "hello");
    }

    #[test]
    fn task_lifecycle() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let base = temp_home("task");
        set_test_env(&base);
        handle_add(&vec![
            "task".into(),
            "Do Stuff".into(),
            "--priority".into(),
            "high".into(),
            "--due".into(),
            "2024-01-01".into(),
        ])
        .unwrap();
        let config = ensure_setup(None).unwrap();
        let tasks = load_items(&config, ItemKind::Task).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, Some(Status::Pending));
        let id = tasks[0].id.clone();
        handle_complete(&vec![id.clone()], true).unwrap();
        let (_k, _p, updated) = resolve_item(&config, &id).unwrap();
        assert_eq!(updated.status, Some(Status::Completed));
    }

    #[test]
    fn tags_are_tracked() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let base = temp_home("tags");
        set_test_env(&base);
        handle_add(&vec![
            "note".into(),
            "Tagged".into(),
            "--tags".into(),
            "one,two".into(),
        ])
        .unwrap();
        let config = ensure_setup(None).unwrap();
        let notes = load_items(&config, ItemKind::Note).unwrap();
        let id = notes[0].id.clone();
        let tag_one = config
            .root
            .join("tags")
            .join("one")
            .join(format!("{id}.lnk"));
        assert!(tag_one.exists());
        handle_edit(&vec![
            id.clone(),
            "--tags".into(),
            "two,three".into(),
            "--title".into(),
            "New".into(),
        ])
        .unwrap();
        let tag_three = config
            .root
            .join("tags")
            .join("three")
            .join(format!("{id}.lnk"));
        assert!(tag_three.exists());
        assert!(!tag_one.exists());
    }
}
