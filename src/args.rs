use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "fumar",
    about = "Control your Storz & Bickel vaporizer from the terminal",
    version
)]
pub struct Cli {
    /// Force TUI mode
    #[arg(long)]
    pub tui: bool,

    /// Force CLI mode
    #[arg(long)]
    pub cli: bool,

    /// Force GUI mode (requires 'gui' feature)
    #[arg(long)]
    pub gui: bool,

    /// Enable Discord Rich Presence
    #[arg(long)]
    pub discord: bool,

    /// BLE scan timeout in seconds
    #[arg(long, default_value = "10")]
    pub scan_timeout: u64,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show target temperature
    Temp,
    /// Set target temperature in Celsius
    SetTemp { celsius: f32 },
    /// Turn heater on
    HeatOn,
    /// Turn heater off
    HeatOff,
    /// Turn pump on (Volcano only)
    PumpOn,
    /// Turn pump off (Volcano only)
    PumpOff,
    /// Show device status as JSON
    Status,
    /// Stream live state updates
    Watch,
}
