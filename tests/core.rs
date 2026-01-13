use mdnotes::config::{ensure_setup, SetupOptions};
use mdnotes::models::{ItemKind, Status};
use mdnotes::storage::{load_items, resolve_item};
use std::fs;
use std::path::PathBuf;

fn temp_home(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join("mdnotes-tests").join(name);
    if base.exists() {
        let _ = fs::remove_dir_all(&base);
    }
    fs::create_dir_all(&base).unwrap();
    base
}

fn base_args(base: &std::path::Path) -> Vec<String> {
    vec![
        "mdn".into(),
        "--config-home".into(),
        base.join("config").to_string_lossy().into(),
        "--root-override".into(),
        base.join("repo").to_string_lossy().into(),
    ]
}

fn run_with(base: &std::path::Path, args: &[&str]) -> Vec<String> {
    let mut full = base_args(base);
    full.extend(args.iter().map(|s| s.to_string()));
    mdnotes::run_with_args(full).expect("command should succeed")
}

#[test]
fn setup_creates_directories_and_config() {
    let base = temp_home("setup");
    let output = run_with(&base, &["setup"]);
    assert!(output.iter().any(|l| l.contains("Config:")));
    assert!(base.join("repo/notes").exists());
    assert!(base.join("repo/tasks").exists());
    assert!(base.join("repo/tags").exists());
}

#[test]
fn add_list_and_show_note() {
    let base = temp_home("note");
    run_with(
        &base,
        &[
            "add",
            "note",
            "My Note",
            "--body",
            "hello",
            "--tags",
            "rust,notes",
        ],
    );
    let list = run_with(&base, &["list", "notes"]);
    assert_eq!(list.len(), 1);
    let show = run_with(&base, &["show", list[0].split(' ').next().unwrap()]);
    assert!(show.iter().any(|l| l.contains("My Note")));
    assert!(show.iter().any(|l| l.contains("rust")));
}

#[test]
fn task_lifecycle_with_due_and_priority() {
    let base = temp_home("task");
    run_with(
        &base,
        &[
            "add",
            "task",
            "Do Stuff",
            "--priority",
            "high",
            "--due",
            "2099-01-01",
        ],
    );
    let config = ensure_setup(SetupOptions {
        root_override: Some(base.join("repo")),
        config_home: Some(base.join("config")),
    })
    .unwrap();
    let tasks = load_items(&config, ItemKind::Task).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].status, Some(Status::Pending));
    let id = tasks[0].id.clone();
    run_with(&base, &["complete", &id]);
    let (_k, _p, updated) = resolve_item(&config, &id).unwrap();
    assert_eq!(updated.status, Some(Status::Completed));
}

#[test]
fn search_finds_note_by_title_and_body() {
    let base = temp_home("search");
    run_with(
        &base,
        &["add", "note", "Alpha Note", "--body", "first body"],
    );
    run_with(
        &base,
        &["add", "note", "Second", "--body", "contains keyword"],
    );
    let results = run_with(&base, &["search", "keyword"]);
    assert_eq!(results.len(), 1);
    assert!(results[0].contains("Second"));
}

#[test]
fn delete_cleans_up_tags() {
    let base = temp_home("delete");
    run_with(&base, &["add", "note", "Tagged", "--tags", "one,two"]);
    let list = run_with(&base, &["list", "notes"]);
    let id = list[0].split(' ').next().unwrap().to_string();
    let tag_one = base.join("repo/tags/one").join(&id);
    assert!(tag_one.exists());
    run_with(&base, &["delete", &id]);
    assert!(!tag_one.exists());
}
