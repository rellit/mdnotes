use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "mdnui", version, about = "mdnotes - interactive TUI")]
struct UiCli {
    /// Override config home (for testing)
    #[arg(long, hide = true)]
    config_home: Option<PathBuf>,
    /// Override root directory (for testing)
    #[arg(long, hide = true)]
    root_override: Option<PathBuf>,
}

fn main() {
    let args = UiCli::parse();
    let setup = mdnotes::config::SetupOptions {
        root_override: args.root_override,
        config_home: args.config_home,
        remote_override: None,
        editor_override: None,
    };
    if let Err(err) = mdnotes::tui::run_tui(setup) {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
