use std::time::Duration;

use anyhow::{Context, Result};
use futures::StreamExt;
use storz_rs::{StorzError, VaporizerControl};
use tracing::debug;

use crate::args::Commands;

pub async fn run(device: Box<dyn VaporizerControl>, cmd: Commands) -> Result<()> {
    // Venty/Veazy send state via notifications. Wait briefly for first update
    // so cached state is populated before we read it.
    wait_for_state(device.as_ref()).await;

    match cmd {
        Commands::Temp => {
            let state = timeout_ble(device.get_state())
                .await
                .context("Failed to get state")?;
            let cur = state
                .current_temp
                .map(|t| format!("{t:.1}\u{b0}C"))
                .unwrap_or_else(|| "---".into());
            let tgt = state
                .target_temp
                .map(|t| format!("{t:.1}\u{b0}C"))
                .unwrap_or_else(|| "---".into());
            println!("Current: {cur}  Target: {tgt}");
        }
        Commands::SetTemp { celsius } => {
            let rounded = (celsius / 2.0).round() * 2.0;
            timeout_ble(device.set_target_temperature(rounded))
                .await
                .context("Failed to set temperature")?;
            println!("Target set to {rounded:.0}\u{b0}C");
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
            let mut json = serde_json::json!({
                "device": device.device_model().to_string(),
                "current_temp": state.current_temp,
                "target_temp": state.target_temp,
                "heater": state.heater_on,
                "setpoint_reached": state.setpoint_reached,
                "pump": state.pump_on,
                "fan": state.fan_on,
            });
            if let Some(ref s) = state.settings {
                json["battery"] = serde_json::json!(s.battery_level);
                json["charging"] = serde_json::json!(s.is_charging);
                json["unit"] = serde_json::json!(if s.is_celsius { "C" } else { "F" });
                json["auto_off_seconds"] = serde_json::json!(s.auto_shutdown_seconds);
            }
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        Commands::Watch => {
            let mut stream = device
                .subscribe_state()
                .await
                .context("Failed to subscribe to state updates")?;
            while let Some(state) = stream.next().await {
                let now = chrono_now();
                let cur = state
                    .current_temp
                    .map(|t| format!("{t:.1}\u{b0}C"))
                    .unwrap_or_else(|| "---".into());
                let tgt = state
                    .target_temp
                    .map(|t| format!("{t:.1}\u{b0}C"))
                    .unwrap_or_else(|| "---".into());
                let heater = if state.heater_on { "ON" } else { "OFF" };
                let pump = if state.pump_on { "ON" } else { "OFF" };
                println!("[{now}]  {cur} / {tgt}  Heater: {heater}  Pump: {pump}");
            }
        }
    }
    Ok(())
}

async fn wait_for_state(device: &dyn VaporizerControl) {
    let mut stream = match device.subscribe_state().await {
        Ok(s) => s,
        Err(e) => {
            debug!("Could not subscribe for initial state: {e}");
            return;
        }
    };

    match tokio::time::timeout(Duration::from_secs(3), stream.next()).await {
        Ok(Some(state)) => {
            debug!(
                "Got initial state: current={:?} target={:?}",
                state.current_temp, state.target_temp
            );
        }
        Ok(None) => debug!("State stream ended before first update"),
        Err(_) => debug!("Timed out waiting for first state notification"),
    }
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
