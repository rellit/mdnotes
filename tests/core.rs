use mdnotes::config::{ensure_setup, save_config, SetupOptions};
use mdnotes::git::sync_pull;
use mdnotes::models::Status;
use mdnotes::storage::{load_all_items, load_items, resolve_item};
use std::fs;
use std::path::{Path, PathBuf};
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

fn git_rev_parse(repo: &Path) -> String {
    let out = Command::new("git")
        .current_dir(repo)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn git_rev_parse_bare(git_dir: &Path) -> String {
    let out = Command::new("git")
        .arg(format!("--git-dir={}", git_dir.to_string_lossy()))
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn config_creates_directories_and_config() {
    let base = temp_home("config");
    let output = run_with(&base, &["config"]);
    assert!(output.iter().any(|l| l.contains("Config:")));
    // The root directory should exist; no notes/tasks/tags subdirs are created upfront
    assert!(base.join("repo").exists());
    assert!(!base.join("repo/notes").exists());
    assert!(!base.join("repo/tasks").exists());
    assert!(!base.join("repo/tags").exists());
}

#[test]
fn add_list_and_show_note() {
    let base = temp_home("note");
    run_with(
        &base,
        &["add", "My Note", "--body", "hello", "--tags", "rust,notes"],
    );
    // List all items (no filter)
    let list = run_with(&base, &["list"]);
    assert_eq!(list.len(), 1);
    assert!(list[0].contains("My Note"));
    // List only tasks: note has no due date, so list should be empty
    let tasks_only = run_with(&base, &["list", ".task"]);
    assert_eq!(tasks_only.len(), 0);
    let id = list[0].split(' ').next().unwrap();
    let show = run_with(&base, &["show", id]);
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
    let tasks = load_all_items(&config).unwrap();
    assert_eq!(tasks.len(), 1);
    assert!(tasks[0].is_task());
    assert_eq!(tasks[0].status, Some(Status::Pending));
    let id = tasks[0].id.clone();
    run_with(&base, &["complete", &id]);
    let (_p, updated) = resolve_item(&config, &id).unwrap();
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
    let all = load_all_items(&config).unwrap();
    assert_eq!(all.len(), 1);
    assert!(!all[0].is_task());
    let id = all[0].id.clone();
    run_with(&base, &["due", &id, "2099-02-02"]);
    let (_p, updated) = resolve_item(&config, &id).unwrap();
    assert_eq!(updated.due, Some("2099-02-02".into()));
    assert!(updated.is_task());
    assert_eq!(updated.status, Some(Status::Pending));
    run_with(&base, &["due", &id]);
    let (_p2, cleared) = resolve_item(&config, &id).unwrap();
    assert_eq!(cleared.due, None);
    assert!(!cleared.is_task());
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
    let all = load_all_items(&config).unwrap();
    assert_eq!(all.len(), 1);
    assert!(!all[0].is_task());
    assert_eq!(all[0].priority, Some(mdnotes::Priority::High));
}

#[test]
fn list_items_with_query_filter() {
    let base = temp_home("list_query");
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
    // No filter: both items returned
    let all = run_with(&base, &["list"]);
    assert_eq!(all.len(), 2);

    // Filter: only tasks
    let tasks = run_with(&base, &["list", ".task"]);
    assert_eq!(tasks.len(), 1);
    assert!(tasks[0].contains("Task One"));

    // Filter: only notes (not tasks)
    let notes = run_with(&base, &["list", ".task not"]);
    assert_eq!(notes.len(), 1);
    assert!(notes[0].contains("Note One"));

    // Filter by priority
    let low_prio = run_with(&base, &["list", "prio:low"]);
    assert_eq!(low_prio.len(), 1);
    assert!(low_prio[0].contains("Task One"));
}

#[test]
fn list_query_tag_filter() {
    let base = temp_home("list_tags");
    run_with(&base, &["add", "Tagged Note", "--tags", "alpha"]);
    run_with(&base, &["add", "Other Note"]);

    let tagged = run_with(&base, &["list", "#alpha"]);
    assert_eq!(tagged.len(), 1);
    assert!(tagged[0].contains("Tagged Note"));
}

#[test]
fn list_query_due_filter() {
    let base = temp_home("list_due");
    run_with(&base, &["add", "Early Task", "--due", "2099-01-01"]);
    run_with(&base, &["add", "Late Task", "--due", "2099-12-31"]);

    // due:> 2099-06-01 → only Late Task
    let late = run_with(&base, &["list", "due:>20990601"]);
    assert_eq!(late.len(), 1);
    assert!(late[0].contains("Late Task"));

    // due:< 2099-06-01 → only Early Task
    let early = run_with(&base, &["list", "due:<20990601"]);
    assert_eq!(early.len(), 1);
    assert!(early[0].contains("Early Task"));
}

#[test]
fn list_query_and_or() {
    let base = temp_home("list_bool");
    run_with(
        &base,
        &["add", "Alpha Task", "--due", "2099-01-01", "--tags", "alpha"],
    );
    run_with(&base, &["add", "Beta Task", "--due", "2099-06-01"]);
    run_with(&base, &["add", "Alpha Note", "--tags", "alpha"]);

    // .task #alpha and → tasks with tag alpha
    let t = run_with(&base, &["list", ".task #alpha and"]);
    assert_eq!(t.len(), 1);
    assert!(t[0].contains("Alpha Task"));

    // .task #alpha or → tasks OR items with tag alpha
    let t = run_with(&base, &["list", ".task #alpha or"]);
    assert_eq!(t.len(), 3);
}

#[test]
fn edit_without_fields_opens_editor_and_sets_due() {
    let base = temp_home("edit_editor");
    run_with(&base, &["add", "Draft"]);
    let config = ensure_setup(SetupOptions {
        root_override: Some(base.join("repo")),
        config_home: Some(base.join("config")),
        remote_override: None,
        editor_override: None,
    })
    .unwrap();
    let all = load_all_items(&config).unwrap();
    let id = all[0].id.clone();
    let main_path = base.join("repo").join(&id).join("MAIN.md");
    let mut content = std::fs::read_to_string(&main_path).unwrap();
    // Insert due/status headers before the "--" body separator
    if let Some(pos) = content.find("\n--\n") {
        content.insert_str(pos, "\ndue: 2099-05-01\nstatus: pending");
    }
    std::fs::write(&main_path, content).unwrap();
    let prev_editor = std::env::var("EDITOR").ok();
    std::env::set_var("EDITOR", "true");
    run_with(&base, &["edit", &id]);
    if let Some(prev) = prev_editor {
        std::env::set_var("EDITOR", prev);
    } else {
        std::env::remove_var("EDITOR");
    }
    // File should still be in the same UUID directory
    assert!(main_path.exists());
    let (_p, updated) = resolve_item(&config, &id).unwrap();
    assert!(updated.is_task());
    assert_eq!(updated.due, Some("2099-05-01".into()));
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
    let all = load_all_items(&config).unwrap();
    let id = all[0].id.clone();
    let new_id = "manually-changed-id";
    let main_path = base.join("repo").join(&id).join("MAIN.md");
    let mut content = std::fs::read_to_string(&main_path).unwrap();
    content = content.replace(&format!("id: {}", id), &format!("id: {}", new_id));
    std::fs::write(&main_path, content).unwrap();
    let prev_editor = std::env::var("EDITOR").ok();
    std::env::set_var("EDITOR", "true");
    run_with(&base, &["edit", &id]);
    if let Some(prev) = prev_editor {
        std::env::set_var("EDITOR", prev);
    } else {
        std::env::remove_var("EDITOR");
    }
    // The UUID dir and file should still exist with the original id
    assert!(main_path.exists());
    let (_p, updated) = resolve_item(&config, &id).unwrap();
    assert_eq!(updated.id, id);
    let updated_content = std::fs::read_to_string(&main_path).unwrap();
    assert!(updated_content.contains(&format!("id: {}", id)));
}

#[test]
fn find_searches_all_items() {
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
    // find without target: all items matching the query, flat list
    let results = run_with(&base, &["find", "keyword"]);
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|r| r.contains("Alpha Note")));
    assert!(results.iter().any(|r| r.contains("Second")));

    // find with tasks target
    let task_only = run_with(&base, &["find", "keyword", "t"]);
    assert_eq!(task_only.len(), 1);
    assert!(task_only[0].contains("Second"));

    // find with notes target
    let notes_only = run_with(&base, &["find", "keyword", "n"]);
    assert_eq!(notes_only.len(), 1);
    assert!(notes_only[0].contains("Alpha Note"));
}

#[test]
fn delete_removes_item_directory() {
    let base = temp_home("delete");
    run_with(&base, &["add", "Tagged", "--tags", "one,two"]);
    let list = run_with(&base, &["list"]);
    let id = list[0].split(' ').next().unwrap().to_string();
    let item_dir = base.join("repo").join(&id);
    assert!(item_dir.exists());
    run_with(&base, &["delete", &id]);
    assert!(!item_dir.exists());
    // No more tag index files to check since indexing is removed
}

#[test]
fn sync_pull_fast_forwards_remote_updates() {
    let base = temp_home("sync_pull_ff");
    let remote = base.join("remote.git");
    let remote_str = remote.to_string_lossy().to_string();
    assert!(Command::new("git")
        .args(["init", "--bare", &remote_str])
        .output()
        .unwrap()
        .status
        .success());

    run_with(&base, &["config", "--remote", &remote_str]);
    run_with(&base, &["add", "Initial"]);

    let config = ensure_setup(SetupOptions {
        root_override: Some(base.join("repo")),
        config_home: Some(base.join("config")),
        remote_override: None,
        editor_override: None,
    })
    .unwrap();
    let initial_head = git_rev_parse(&config.root);

    let clone_dir = base.join("clone");
    assert!(Command::new("git")
        .args(["clone", &remote_str, clone_dir.to_string_lossy().as_ref()])
        .output()
        .unwrap()
        .status
        .success());
    assert!(Command::new("git")
        .current_dir(&clone_dir)
        .args(["config", "user.email", "tester@example.com"])
        .output()
        .unwrap()
        .status
        .success());
    assert!(Command::new("git")
        .current_dir(&clone_dir)
        .args(["config", "user.name", "Tester"])
        .output()
        .unwrap()
        .status
        .success());
    fs::write(clone_dir.join("README.md"), "update").unwrap();
    assert!(Command::new("git")
        .current_dir(&clone_dir)
        .args(["add", "README.md"])
        .output()
        .unwrap()
        .status
        .success());
    assert!(Command::new("git")
        .current_dir(&clone_dir)
        .args(["commit", "-m", "remote update"])
        .output()
        .unwrap()
        .status
        .success());
    assert!(Command::new("git")
        .current_dir(&clone_dir)
        .args(["push"])
        .output()
        .unwrap()
        .status
        .success());

    let before_head = git_rev_parse(&config.root);
    assert_eq!(before_head, initial_head);

    sync_pull(&config).unwrap();

    let after_head = git_rev_parse(&config.root);
    assert_ne!(after_head, before_head);
    let remote_head = git_rev_parse_bare(&remote);
    assert_eq!(after_head, remote_head);
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

#[test]
fn load_items_filters_by_kind() {
    let base = temp_home("load_items_kind");
    run_with(&base, &["add", "Plain Note"]);
    run_with(&base, &["add", "A Task", "--due", "2099-03-15"]);
    let config = ensure_setup(SetupOptions {
        root_override: Some(base.join("repo")),
        config_home: Some(base.join("config")),
        remote_override: None,
        editor_override: None,
    })
    .unwrap();
    let notes = load_items(&config, mdnotes::ItemKind::Note).unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].title, "Plain Note");
    let tasks = load_items(&config, mdnotes::ItemKind::Task).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "A Task");
}

