mod args;
mod cli;
mod config;
#[cfg(feature = "discord")]
mod discord;
mod scanner;
mod tui;

#[cfg(feature = "gui")]
mod gui;

use std::io::IsTerminal;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tracing::debug;

use args::{Cli, Commands};
use config::Config;
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

fn init_tracing(gui_mode: bool) {
    if gui_mode {
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
    let cfg = Config::load().unwrap_or_else(|e| {
        debug!("Failed to load config: {e}, using defaults");
        Config::default()
    });

    // Determine effective values: CLI flags override config
    let use_discord = args.discord || cfg.discord;
    let scan_timeout = if args.scan_timeout != 10 {
        args.scan_timeout
    } else {
        cfg.scan_timeout
    };
    let mode = if args.gui {
        "gui"
    } else if args.tui {
        "tui"
    } else if args.cli {
        "cli"
    } else {
        &cfg.mode
    };

    // Handle config command before BLE connection
    if matches!(args.command, Some(Commands::Config)) {
        let path = Config::config_path()?;
        eprintln!("Config file: {}\n", path.display());
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            println!("{content}");
        } else {
            eprintln!("(not yet created — run fumar to set up)");
        }
        return Ok(());
    }

    if mode == "gui" {
        init_tracing(true);
        #[cfg(feature = "gui")]
        {
            #[cfg(feature = "discord")]
            if use_discord {
                discord::init();
            }
            gui::run_gui();
            #[cfg(feature = "discord")]
            if use_discord {
                discord::clear();
            }
            return Ok(());
        }
        #[cfg(not(feature = "gui"))]
        {
            anyhow::bail!("GUI mode requires: cargo install fumar --features gui");
        }
    }

    let tui_mode = mode == "tui"
        || (mode != "cli" && args.command.is_none() && stdout_is_tty());

    init_tracing(false);

    let timeout = Duration::from_secs(scan_timeout);
    let device = scanner::scan_and_select(timeout).await?;

    #[cfg(feature = "discord")]
    if use_discord {
        discord::init();
    }

    if tui_mode {
        let _guard = TerminalGuard::new()?;
        let backend = CrosstermBackend::new(std::io::stdout());
        let mut terminal = Terminal::new(backend)?;
        let mut app = App::new(device).await;
        tui::events::run(&mut app, &mut terminal).await?;
        #[cfg(feature = "discord")]
        if use_discord {
            discord::clear();
        }
    } else {
        let cmd = args.command.unwrap_or(Commands::Status);
        cli::run(device, cmd).await?;
        #[cfg(feature = "discord")]
        if use_discord {
            discord::clear();
        }
    }

    Ok(())
}
