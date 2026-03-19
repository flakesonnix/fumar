use std::time::Duration;

use anyhow::{Context, Result};
use futures::StreamExt;
use storz_rs::{StorzError, VaporizerControl};

use crate::args::Commands;

pub async fn run(device: Box<dyn VaporizerControl>, cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Temp => {
            let current = match timeout_ble(device.get_current_temperature()).await {
                Ok(t) => format!("{t:.1}°C"),
                Err(_) => "N/A".into(),
            };
            let target = timeout_ble(device.get_target_temperature()).await?;
            println!("Current: {current}  Target: {target:.1}°C");
        }
        Commands::SetTemp { celsius } => {
            let rounded = (celsius / 2.0).round() * 2.0;
            timeout_ble(device.set_target_temperature(rounded))
                .await
                .context("Failed to set temperature")?;
            println!("Target set to {rounded:.0}°C");
        }
        Commands::HeatOn => {
            timeout_ble(device.heater_on())
                .await
                .context("Failed to turn heater on")?;
            println!("Heater on");
        }
        Commands::HeatOff => {
            timeout_ble(device.heater_off())
                .await
                .context("Failed to turn heater off")?;
            println!("Heater off");
        }
        Commands::PumpOn => match timeout_ble(device.pump_on()).await {
            Ok(_) => println!("Pump on"),
            Err(e) => {
                if is_unsupported(&e) {
                    eprintln!("Not supported on {}", device.device_model());
                    std::process::exit(2);
                }
                return Err(e);
            }
        },
        Commands::PumpOff => match timeout_ble(device.pump_off()).await {
            Ok(_) => println!("Pump off"),
            Err(e) => {
                if is_unsupported(&e) {
                    eprintln!("Not supported on {}", device.device_model());
                    std::process::exit(2);
                }
                return Err(e);
            }
        },
        Commands::Status => {
            let state = timeout_ble(device.get_state())
                .await
                .context("Failed to get state")?;
            let json = serde_json::json!({
                "device": device.device_model().to_string(),
                "current_temp": state.current_temp,
                "target_temp": state.target_temp,
                "heater": state.heater_on,
                "pump": state.pump_on,
                "fan": state.fan_on,
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        Commands::Watch => {
            let mut stream = device
                .subscribe_state()
                .await
                .context("Failed to subscribe to state updates")?;
            let mut count = 0u32;
            while let Some(state) = stream.next().await {
                let now = chrono_now();
                let cur = state
                    .current_temp
                    .map(|t| format!("{t:.1}°C"))
                    .unwrap_or_else(|| "---".into());
                let tgt = state
                    .target_temp
                    .map(|t| format!("{t:.1}°C"))
                    .unwrap_or_else(|| "---".into());
                let heater = if state.heater_on { "ON" } else { "OFF" };
                let pump = if state.pump_on { "ON" } else { "OFF" };
                println!("[{now}]  {cur} / {tgt}  Heater: {heater}  Pump: {pump}");
                count += 1;
                if count >= 200 {
                    break;
                }
            }
        }
    }
    Ok(())
}

fn chrono_now() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs() % 86400;
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

async fn timeout_ble<F, T>(fut: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T, StorzError>>,
{
    tokio::time::timeout(Duration::from_secs(5), fut)
        .await
        .context("BLE operation timed out")?
        .map_err(|e| anyhow::anyhow!("{e}"))
}

fn is_unsupported(err: &anyhow::Error) -> bool {
    err.to_string().contains("Unsupported operation")
}
