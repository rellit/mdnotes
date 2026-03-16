use crate::config::Config;
use crate::{MdError, MdResult};
use std::path::Path;
use std::process::{Command, Output};

fn git(root: &Path, args: &[&str]) -> MdResult<Output> {
    Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .map_err(|e| MdError(e.to_string()))
}

fn current_branch(root: &Path) -> MdResult<String> {
    let out = git(root, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    if out.status.success() {
        let branch = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !branch.is_empty() && branch != "HEAD" {
            return Ok(branch);
        }
    }
    if let Some(default) = default_branch_from_config(root)? {
        return Ok(default);
    }
    Ok("main".into())
}

fn default_branch_from_config(root: &Path) -> MdResult<Option<String>> {
    let out = git(root, &["config", "--get", "init.defaultBranch"])?;
    if out.status.success() {
        let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !val.is_empty() {
            return Ok(Some(val));
        }
    }
    Ok(None)
}

fn remote_name(root: &Path) -> MdResult<String> {
    let out = git(root, &["remote"])?;
    if out.status.success() {
        let mut first: Option<String> = None;
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let name = line.trim();
            if name.is_empty() {
                continue;
            }
            if first.is_none() {
                first = Some(name.to_string());
            }
            if name == "origin" {
                return Ok("origin".into());
            }
        }
        if let Some(fallback) = first {
            return Ok(fallback);
        }
    }
    Ok("origin".into())
}

pub fn sync_pull(config: &Config) -> MdResult<()> {
    if config.remote.is_none() {
        return Ok(());
    }
    eprintln!("Checking for remote updates...");
    let remote_name = remote_name(&config.root)?;
    let branch = current_branch(&config.root)?;
    let fetch = git(&config.root, &["fetch", "--quiet", &remote_name])?;
    if !fetch.status.success() {
        return Err(MdError(format!(
            "git fetch failed: {}",
            String::from_utf8_lossy(&fetch.stderr)
        )));
    }
    let remote_ref = format!("{}/{}", remote_name, branch);
    let verify = git(&config.root, &["rev-parse", "--verify", &remote_ref])?;
    if !verify.status.success() {
        return Ok(());
    }
    let head = git(&config.root, &["rev-parse", "HEAD"])?;
    if !head.status.success() {
        return Err(MdError(format!(
            "git rev-parse HEAD failed: {}",
            String::from_utf8_lossy(&head.stderr)
        )));
    }
    let local_head = String::from_utf8_lossy(&head.stdout).trim().to_string();
    let remote_head = git(&config.root, &["rev-parse", &remote_ref])?;
    if !remote_head.status.success() {
        return Err(MdError(format!(
            "git rev-parse {remote_ref} failed: {}",
            String::from_utf8_lossy(&remote_head.stderr)
        )));
    }
    let remote_head = String::from_utf8_lossy(&remote_head.stdout)
        .trim()
        .to_string();
    if local_head == remote_head {
        return Ok(());
    }
    let merge = git(&config.root, &["merge", "--ff-only", &remote_ref])?;
    if !merge.status.success() {
        return Err(MdError(format!(
            "git merge failed: {}",
            String::from_utf8_lossy(&merge.stderr)
        )));
    }
    Ok(())
}

pub fn sync_push(config: &Config, message: &str) -> MdResult<()> {
    if !has_changes(&config.root)? {
        return Ok(());
    }
    ensure_user_identity(&config.root)?;
    let add = git(&config.root, &["add", "-A"])?;
    if !add.status.success() {
        return Err(MdError(format!(
            "git add failed: {}",
            String::from_utf8_lossy(&add.stderr)
        )));
    }
    let commit = git(&config.root, &["commit", "-m", message])?;
    if !commit.status.success() {
        return Err(MdError(format!(
            "git commit failed: {}",
            String::from_utf8_lossy(&commit.stderr)
        )));
    }
    if config.remote.is_none() {
        return Ok(());
    }
    let remote_name = remote_name(&config.root)?;
    let branch = current_branch(&config.root)?;
    let mut push_args = vec!["push"];
    if !upstream_configured(&config.root)? {
        push_args.push("-u");
    }
    push_args.push(&remote_name);
    push_args.push(&branch);
    let push = git(&config.root, &push_args)?;
    if !push.status.success() {
        let stderr = String::from_utf8_lossy(&push.stderr);
        if stderr.contains("rejected") {
            return Err(MdError(format!(
                "git push rejected: remote has changes not present locally.\n\
                 Run 'mdnotes sync' to pull and merge remote changes first.\n\
                 {}",
                stderr.trim()
            )));
        }
        return Err(MdError(format!("git push failed: {}", stderr)));
    }
    Ok(())
}

fn has_changes(root: &Path) -> MdResult<bool> {
    let status = git(root, &["status", "--porcelain"])?;
    if !status.status.success() {
        return Err(MdError(format!(
            "git status failed: {}",
            String::from_utf8_lossy(&status.stderr)
        )));
    }
    Ok(!status.stdout.is_empty())
}

fn upstream_configured(root: &Path) -> MdResult<bool> {
    let out = git(
        root,
        &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
    )?;
    Ok(out.status.success())
}

fn ensure_user_identity(root: &Path) -> MdResult<()> {
    let default_name = std::env::var("GIT_AUTHOR_NAME")
        .or_else(|_| std::env::var("GIT_COMMITTER_NAME"))
        .unwrap_or_else(|_| "mdnotes".into());
    let default_email = std::env::var("GIT_AUTHOR_EMAIL")
        .or_else(|_| std::env::var("GIT_COMMITTER_EMAIL"))
        .unwrap_or_else(|_| "mdnotes@example.com".into());
    ensure_git_config(root, "user.name", &default_name)?;
    ensure_git_config(root, "user.email", &default_email)?;
    Ok(())
}

fn ensure_git_config(root: &Path, key: &str, value: &str) -> MdResult<()> {
    let current = git(root, &["config", key])?;
    if current.status.success() && !String::from_utf8_lossy(&current.stdout).trim().is_empty() {
        return Ok(());
    }
    let set = git(root, &["config", key, value])?;
    if !set.status.success() {
        return Err(MdError(format!(
            "git config {key} failed: {}",
            String::from_utf8_lossy(&set.stderr)
        )));
    }
    Ok(())
}
