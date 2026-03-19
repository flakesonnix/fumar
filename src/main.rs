mod args;
mod cli;
mod scanner;
mod tui;

use std::io::IsTerminal;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use args::{Cli, Commands};
use tui::app::App;

struct TerminalGuard;

impl TerminalGuard {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        execute!(std::io::stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
    }
}

fn init_tracing(tui_mode: bool) {
    if tui_mode {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/fumar.log")
            .expect("cannot open log file");
        tracing_subscriber::fmt()
            .with_writer(file)
            .with_env_filter("debug")
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }
}

fn stdout_is_tty() -> bool {
    std::io::stdout().is_terminal()
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let tui_mode = args.tui || (!args.cli && args.command.is_none() && stdout_is_tty());

    init_tracing(tui_mode);

    let timeout = Duration::from_secs(args.scan_timeout);
    let device = scanner::scan_and_select(timeout).await?;

    if tui_mode {
        let _guard = TerminalGuard::new()?;
        let backend = CrosstermBackend::new(std::io::stdout());
        let mut terminal = Terminal::new(backend)?;
        let mut app = App::new(device).await;
        tui::events::run(&mut app, &mut terminal).await?;
    } else {
        let cmd = args.command.unwrap_or(Commands::Status);
        cli::run(device, cmd).await?;
    }

    Ok(())
}
