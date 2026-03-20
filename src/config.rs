use std::io::BufRead;
use std::path::PathBuf;

use anyhow::{Context, Result};
use kdl::{KdlDocument, KdlNode, KdlValue};
use tracing::debug;

/// Persistent configuration loaded from `~/.config/fumar/config.kdl`.
#[derive(Debug, Clone)]
pub struct Config {
    /// Default UI mode: "tui", "cli", or "gui".
    pub mode: String,
    /// Enable Discord Rich Presence.
    pub discord: bool,
    /// BLE scan timeout in seconds.
    pub scan_timeout: u64,
    /// Default target temperature in Celsius.
    pub default_temp: f32,
    /// Auto-connect to first found device.
    pub auto_connect: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: "tui".into(),
            discord: false,
            scan_timeout: 10,
            default_temp: 180.0,
            auto_connect: false,
        }
    }
}

impl Config {
    /// Path to the config file.
    pub fn config_path() -> Result<PathBuf> {
        let base = dirs_fallback();
        Ok(base.join("fumar").join("config.kdl"))
    }

    /// Load config from disk, creating with interactive prompts if missing.
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            debug!("Config not found at {path:?}, starting first-run setup");
            let config = Self::interactive_setup()?;
            config.save()?;
            eprintln!("\nConfig saved to {}\n", path.display());
            return Ok(config);
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {path:?}"))?;

        Self::parse(&content)
    }

    /// Interactive first-run setup — prompts user for each setting.
    fn interactive_setup() -> Result<Self> {
        use std::io::{self, Write};

        eprintln!("Welcome to fumar! Let's set up your config.\n");

        let stdin = io::stdin();
        let mut lines = stdin.lock();

        let mut config = Self::default();

        // Mode
        eprintln!("Default UI mode:");
        eprintln!("  [1] TUI  (terminal interface, default)");
        eprintln!("  [2] CLI  (command-line only)");
        eprintln!("  [3] GUI  (graphical, requires GTK4)");
        eprint!("Choice [1]: ");
        io::stderr().flush()?;
        config.mode = match read_line(&mut lines).as_str() {
            "2" => "cli".into(),
            "3" => "gui".into(),
            _ => "tui".into(),
        };
        eprintln!();

        // Discord
        eprint!("Enable Discord Rich Presence? [y/N]: ");
        io::stderr().flush()?;
        config.discord = read_line(&mut lines).to_lowercase().starts_with('y');
        eprintln!();

        // Scan timeout
        eprint!("BLE scan timeout in seconds [10]: ");
        io::stderr().flush()?;
        let input = read_line(&mut lines);
        if let Ok(val) = input.parse::<u64>() {
            config.scan_timeout = val.clamp(1, 120);
        }
        eprintln!();

        // Default temp
        eprint!("Default target temperature in °C [180]: ");
        io::stderr().flush()?;
        let input = read_line(&mut lines);
        if let Ok(val) = input.parse::<f32>() {
            config.default_temp = val.clamp(40.0, 230.0);
        }
        eprintln!();

        // Auto-connect
        eprint!("Auto-connect to first device found? [y/N]: ");
        io::stderr().flush()?;
        config.auto_connect = read_line(&mut lines).to_lowercase().starts_with('y');
        eprintln!();

        eprintln!("Configuration complete!\n");

        Ok(config)
    }

    /// Parse a KDL string into a Config.
    fn parse(content: &str) -> Result<Self> {
        let doc: KdlDocument = content.parse().context("Failed to parse config.kdl")?;

        let mut config = Self::default();

        // Parse "mode" node
        if let Some(node) = doc.get("mode")
            && let Some(val) = node.get(0).and_then(|v| v.as_string())
        {
            config.mode = val.to_string();
        }

        // Parse "discord" node
        if let Some(node) = doc.get("discord")
            && let Some(val) = node.get(0).and_then(|v| v.as_bool())
        {
            config.discord = val;
        }

        // Parse "scan_timeout" node
        if let Some(node) = doc.get("scan_timeout")
            && let Some(val) = node.get(0).and_then(|v| v.as_integer())
        {
            config.scan_timeout = val.clamp(1, 120) as u64;
        }

        // Parse "default_temp" node
        if let Some(node) = doc.get("default_temp")
            && let Some(val) = node.get(0).and_then(|v| v.as_float())
        {
            config.default_temp = (val as f32).clamp(40.0, 230.0);
        }

        // Parse "auto_connect" node
        if let Some(node) = doc.get("auto_connect")
            && let Some(val) = node.get(0).and_then(|v| v.as_bool())
        {
            config.auto_connect = val;
        }

        debug!("Loaded config: {config:?}");
        Ok(config)
    }

    /// Save config to disk as KDL.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config dir {parent:?}"))?;
        }

        let content = self.to_kdl();
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write config to {path:?}"))?;

        debug!("Saved config to {path:?}");
        Ok(())
    }

    /// Serialize config to KDL string.
    fn to_kdl(&self) -> String {
        let mut doc = KdlDocument::new();

        let mut mode_node = KdlNode::new("mode");
        mode_node.push(self.mode.clone());
        doc.nodes_mut().push(mode_node);

        let mut discord_node = KdlNode::new("discord");
        discord_node.push(KdlValue::Bool(self.discord));
        doc.nodes_mut().push(discord_node);

        let mut timeout_node = KdlNode::new("scan_timeout");
        timeout_node.push(self.scan_timeout as i128);
        doc.nodes_mut().push(timeout_node);

        let mut temp_node = KdlNode::new("default_temp");
        temp_node.push(self.default_temp as f64);
        doc.nodes_mut().push(temp_node);

        let mut auto_node = KdlNode::new("auto_connect");
        auto_node.push(KdlValue::Bool(self.auto_connect));
        doc.nodes_mut().push(auto_node);

        // Auto-format for readability
        doc.format();

        doc.to_string()
    }
}

/// Read a trimmed line from stdin, returning empty string on EOF.
fn read_line(reader: &mut impl BufRead) -> String {
    let mut buf = String::new();
    if reader.read_line(&mut buf).is_ok() {
        buf.trim().to_string()
    } else {
        String::new()
    }
}

/// Get config directory, falling back if `dirs` crate not available.
fn dirs_fallback() -> PathBuf {
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
    {
        return dir;
    }

    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".config");
    }

    // Windows fallback
    if let Some(appdata) = std::env::var_os("APPDATA") {
        return PathBuf::from(appdata);
    }

    PathBuf::from(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.mode, "tui");
        assert!(!config.discord);
        assert_eq!(config.scan_timeout, 10);
        assert_eq!(config.default_temp, 180.0);
        assert!(!config.auto_connect);
    }

    #[test]
    fn test_parse_kdl() {
        let kdl = r#"
mode "cli"
discord #true
scan_timeout 15
default_temp 200.0
auto_connect #true
"#;
        let config = Config::parse(kdl).unwrap();
        assert_eq!(config.mode, "cli");
        assert!(config.discord);
        assert_eq!(config.scan_timeout, 15);
        assert_eq!(config.default_temp, 200.0);
        assert!(config.auto_connect);
    }

    #[test]
    fn test_parse_partial() {
        let kdl = r#"
mode "gui"
"#;
        let config = Config::parse(kdl).unwrap();
        assert_eq!(config.mode, "gui");
        // rest should be defaults
        assert!(!config.discord);
        assert_eq!(config.scan_timeout, 10);
    }

    #[test]
    fn test_roundtrip() {
        let config = Config {
            mode: "cli".into(),
            discord: true,
            scan_timeout: 20,
            default_temp: 195.0,
            auto_connect: true,
        };
        let kdl = config.to_kdl();
        let parsed = Config::parse(&kdl).unwrap();
        assert_eq!(parsed.mode, config.mode);
        assert_eq!(parsed.discord, config.discord);
        assert_eq!(parsed.scan_timeout, config.scan_timeout);
        assert_eq!(parsed.default_temp, config.default_temp);
        assert_eq!(parsed.auto_connect, config.auto_connect);
    }

    #[test]
    fn test_clamp_timeout() {
        let kdl = "scan_timeout 999";
        let config = Config::parse(kdl).unwrap();
        assert_eq!(config.scan_timeout, 120);

        let kdl = "scan_timeout 0";
        let config = Config::parse(kdl).unwrap();
        assert_eq!(config.scan_timeout, 1);
    }

    #[test]
    fn test_clamp_temp() {
        let kdl = "default_temp 300.0";
        let config = Config::parse(kdl).unwrap();
        assert_eq!(config.default_temp, 230.0);

        let kdl = "default_temp 10.0";
        let config = Config::parse(kdl).unwrap();
        assert_eq!(config.default_temp, 40.0);
    }
}
