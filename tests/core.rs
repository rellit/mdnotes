use mdnotes::config::{ensure_setup, save_config, SetupOptions};
use mdnotes::models::{ItemKind, Status};
use mdnotes::storage::{load_items, resolve_item};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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
fn config_creates_directories_and_config() {
    let base = temp_home("config");
    let output = run_with(&base, &["config"]);
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
        &["add", "My Note", "--body", "hello", "--tags", "rust,notes"],
    );
    let list = run_with(&base, &["list", "n"]);
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
        remote_override: None,
        editor_override: None,
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
fn due_command_sets_and_clears_due_dates() {
    let base = temp_home("due_command");
    run_with(&base, &["add", "Schedule later"]);
    let config = ensure_setup(SetupOptions {
        root_override: Some(base.join("repo")),
        config_home: Some(base.join("config")),
        remote_override: None,
        editor_override: None,
    })
    .unwrap();
    let notes = load_items(&config, ItemKind::Note).unwrap();
    let id = notes[0].id.clone();
    run_with(&base, &["due", &id, "2099-02-02"]);
    let (_kind, _p, updated) = resolve_item(&config, &id).unwrap();
    assert_eq!(updated.due, Some("2099-02-02".into()));
    assert_eq!(updated.kind, ItemKind::Task);
    assert_eq!(updated.status, Some(Status::Pending));
    run_with(&base, &["due", &id]);
    let (_kind2, _p2, cleared) = resolve_item(&config, &id).unwrap();
    assert_eq!(cleared.due, None);
}

#[test]
fn notes_allow_priority_without_becoming_tasks() {
    let base = temp_home("note_priority");
    run_with(&base, &["add", "Important Note", "--priority", "high"]);
    let config = ensure_setup(SetupOptions {
        root_override: Some(base.join("repo")),
        config_home: Some(base.join("config")),
        remote_override: None,
        editor_override: None,
    })
    .unwrap();
    let notes = load_items(&config, ItemKind::Note).unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].priority, Some(mdnotes::Priority::High));
}

#[test]
fn list_separates_notes_and_tasks_when_no_target() {
    let base = temp_home("list_all");
    run_with(&base, &["add", "Note One"]);
    run_with(
        &base,
        &[
            "add",
            "Task One",
            "--due",
            "2099-12-31",
            "--priority",
            "low",
        ],
    );
    let list = run_with(&base, &["list"]);
    assert_eq!(list[0], "Notes:");
    assert!(list[1].contains("Note One"));
    assert_eq!(list[2], "");
    assert_eq!(list[3], "Tasks:");
    assert!(list[4].contains("Task One"));
}

#[test]
fn edit_without_fields_opens_editor_and_reclassifies() {
    let base = temp_home("edit_editor");
    run_with(&base, &["add", "Draft"]);
    let config = ensure_setup(SetupOptions {
        root_override: Some(base.join("repo")),
        config_home: Some(base.join("config")),
        remote_override: None,
        editor_override: None,
    })
    .unwrap();
    let notes = load_items(&config, ItemKind::Note).unwrap();
    let id = notes[0].id.clone();
    let note_path = base.join("repo/notes").join(format!("{}.md", id));
    let mut content = std::fs::read_to_string(&note_path).unwrap();
    content = content.replace(
        "type: note\n",
        "type: note\nstatus: pending\ndue: 2099-05-01\n",
    );
    std::fs::write(&note_path, content).unwrap();
    let prev_editor = std::env::var("EDITOR").ok();
    std::env::set_var("EDITOR", "true");
    run_with(&base, &["edit", &id]);
    if let Some(prev) = prev_editor {
        std::env::set_var("EDITOR", prev);
    } else {
        std::env::remove_var("EDITOR");
    }
    assert!(!note_path.exists());
    let task_path = base.join("repo/tasks").join(format!("{}.md", id));
    assert!(task_path.exists());
    let (_kind, _p, updated) = resolve_item(&config, &id).unwrap();
    assert_eq!(updated.kind, ItemKind::Task);
}

#[test]
fn edit_restores_changed_id_to_filename_value() {
    let base = temp_home("edit_id_restore");
    run_with(&base, &["add", "Keep ID"]);
    let config = ensure_setup(SetupOptions {
        root_override: Some(base.join("repo")),
        config_home: Some(base.join("config")),
        remote_override: None,
        editor_override: None,
    })
    .unwrap();
    let notes = load_items(&config, ItemKind::Note).unwrap();
    let id = notes[0].id.clone();
    let new_id = "manually-changed-id";
    let note_path = base.join("repo/notes").join(format!("{}.md", id));
    let mut content = std::fs::read_to_string(&note_path).unwrap();
    content = content.replace(&format!("id: {}", id), &format!("id: {}", new_id));
    std::fs::write(&note_path, content).unwrap();
    let prev_editor = std::env::var("EDITOR").ok();
    std::env::set_var("EDITOR", "true");
    run_with(&base, &["edit", &id]);
    if let Some(prev) = prev_editor {
        std::env::set_var("EDITOR", prev);
    } else {
        std::env::remove_var("EDITOR");
    }
    assert!(note_path.exists());
    assert!(!base
        .join("repo/notes")
        .join(format!("{}.md", new_id))
        .exists());
    let (_kind, _p, updated) = resolve_item(&config, &id).unwrap();
    assert_eq!(updated.id, id);
    let updated_content = std::fs::read_to_string(&note_path).unwrap();
    assert!(updated_content.contains(&format!("id: {}", id)));
}

#[test]
fn find_searches_notes_and_tasks() {
    let base = temp_home("find");
    run_with(
        &base,
        &["add", "Alpha Note", "--body", "first keyword body"],
    );
    run_with(
        &base,
        &[
            "add",
            "Second",
            "--body",
            "contains keyword",
            "--due",
            "2099-01-01",
        ],
    );
    let results = run_with(&base, &["find", "keyword"]);
    assert_eq!(results[0], "Notes:");
    assert!(results[1].contains("Alpha Note"));
    assert_eq!(results[2], "");
    assert_eq!(results[3], "Tasks:");
    assert!(results[4].contains("Second"));

    let task_only = run_with(&base, &["find", "keyword", "t"]);
    assert_eq!(task_only.len(), 1);
    assert!(task_only[0].contains("Second"));
}

#[test]
fn delete_cleans_up_tags() {
    let base = temp_home("delete");
    run_with(&base, &["add", "Tagged", "--tags", "one,two"]);
    let list = run_with(&base, &["list", "notes"]);
    let id = list[0].split(' ').next().unwrap().to_string();
    let tag_one = base.join("repo/tags/one").join(&id);
    assert!(tag_one.exists());
    run_with(&base, &["delete", &id]);
    assert!(!tag_one.exists());
}

#[test]
fn config_accepts_remote_and_sets_origin() {
    let base = temp_home("config_remote");
    let remote = "https://example.com/repo.git";
    run_with(&base, &["config", "--remote", remote]);
    let config_contents = std::fs::read_to_string(base.join("config").join("mdnrc")).unwrap();
    assert!(config_contents.contains(remote));
    let origin = Command::new("git")
        .current_dir(base.join("repo"))
        .args(["remote", "get-url", "origin"])
        .output()
        .unwrap();
    assert!(origin.status.success());
    let url = String::from_utf8_lossy(&origin.stdout);
    assert!(url.contains(remote));
}

#[test]
fn save_config_persists_settings() {
    let base = temp_home("config_save");
    let opts = SetupOptions {
        root_override: Some(base.join("repo")),
        config_home: Some(base.join("config")),
        remote_override: None,
        editor_override: None,
    };
    let mut config = ensure_setup(opts.clone()).unwrap();
    config.remote = Some("https://example.com/alt.git".into());
    config.editor = Some("nano".into());
    save_config(&opts, &config).unwrap();
    let reloaded = ensure_setup(opts.clone()).unwrap();
    assert_eq!(
        reloaded.remote.as_deref(),
        Some("https://example.com/alt.git")
    );
    assert_eq!(reloaded.editor.as_deref(), Some("nano"));
}
