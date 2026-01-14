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
    let remote_name = remote_name(&config.root)?;
    let branch = current_branch(&config.root)?;
    let fetch = git(&config.root, &["fetch", &remote_name])?;
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
    let pull = git(
        &config.root,
        &["pull", "--ff-only", &remote_name, &branch],
    )?;
    if !pull.status.success() {
        return Err(MdError(format!(
            "git pull failed: {}",
            String::from_utf8_lossy(&pull.stderr)
        )));
    }
    Ok(())
}

pub fn sync_push(config: &Config, message: &str) -> MdResult<()> {
    if config.remote.is_none() {
        return Ok(());
    }
    let remote_name = remote_name(&config.root)?;
    let branch = current_branch(&config.root)?;
    if !has_changes(&config.root)? {
        return Ok(());
    }
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
    let mut push_args = vec!["push"];
    if !upstream_configured(&config.root)? {
        push_args.push("-u");
    }
    push_args.push(&remote_name);
    push_args.push(&branch);
    let push = git(&config.root, &push_args)?;
    if !push.status.success() {
        return Err(MdError(format!(
            "git push failed: {}",
            String::from_utf8_lossy(&push.stderr)
        )));
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
