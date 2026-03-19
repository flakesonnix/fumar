use std::io::IsTerminal;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use btleplug::api::Peripheral as _;
use storz_rs::VaporizerControl;
use tracing::{info, warn};

pub async fn scan_and_select(timeout: Duration) -> Result<Box<dyn VaporizerControl>> {
    let adapter = storz_rs::get_adapter()
        .await
        .context("Failed to get BLE adapter")?;

    let peripherals = storz_rs::discover_vaporizers(&adapter, timeout)
        .await
        .context("BLE scan failed")?;

    if peripherals.is_empty() {
        bail!("No Storz & Bickel devices found. Make sure your device is powered on and in range.");
    }

    let peripheral = if peripherals.len() == 1 {
        let p = peripherals.into_iter().next().unwrap();
        let name = p
            .properties()
            .await
            .ok()
            .flatten()
            .and_then(|props| props.local_name)
            .unwrap_or_else(|| "Unknown".into());
        eprintln!("Connecting to {name}...");
        p
    } else {
        select_interactive(peripherals).await?
    };

    info!("Connecting to device...");
    let device = tokio::time::timeout(Duration::from_secs(15), storz_rs::connect(peripheral))
        .await
        .context("Connection timed out")?
        .context("Failed to connect")?;

    info!("Connected to {}", device.device_model());
    Ok(device)
}

async fn select_interactive(
    peripherals: Vec<btleplug::platform::Peripheral>,
) -> Result<btleplug::platform::Peripheral> {
    if !std::io::stdout().is_terminal() {
        let p = peripherals.into_iter().next().unwrap();
        warn!("Multiple devices found but stdout is not a TTY, selecting first device");
        return Ok(p);
    }

    eprintln!("\nFound {} devices:", peripherals.len());
    let mut names = Vec::new();
    for (i, p) in peripherals.iter().enumerate() {
        let name = p
            .properties()
            .await
            .ok()
            .flatten()
            .and_then(|props| props.local_name)
            .unwrap_or_else(|| "Unknown".into());
        names.push(name.clone());
        eprintln!("  [{}] {}", i + 1, name);
    }

    eprint!("\nSelect device [1-{}]: ", peripherals.len());
    use std::io::Write;
    std::io::stderr().flush().ok();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let idx = input
        .trim()
        .parse::<usize>()
        .unwrap_or(1)
        .saturating_sub(1)
        .min(peripherals.len() - 1);

    if idx > 0 {
        warn!("Invalid selection, using device [{}]", idx + 1);
    }

    info!("Selected: {}", names[idx]);
    Ok(peripherals.into_iter().nth(idx).unwrap())
}
