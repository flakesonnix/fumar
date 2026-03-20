use std::sync::Mutex;

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use tracing::{debug, warn};

const CLIENT_ID: &str = "1484204070758518927";

static CLIENT: Mutex<Option<DiscordIpcClient>> = Mutex::new(None);

pub fn init() {
    let mut client = DiscordIpcClient::new(CLIENT_ID);
    match client.connect() {
        Ok(()) => {
            debug!("Discord Rich Presence connected");
            CLIENT.lock().unwrap().replace(client);
        }
        Err(e) => {
            warn!("Discord RPC connect failed: {e}");
        }
    }
}

pub fn update(
    model: &str,
    temp: Option<f32>,
    target: Option<f32>,
    heater_on: bool,
    pump_on: bool,
    battery: Option<u8>,
    charging: bool,
) {
    let mut guard = CLIENT.lock().unwrap();
    let Some(client) = guard.as_mut() else {
        return;
    };

    let temp_str = match temp {
        Some(t) => format!("{t:.1}\u{b0}C"),
        None => "---".into(),
    };

    let tgt_str = match target {
        Some(t) => format!("{t:.1}\u{b0}C"),
        None => "---".into(),
    };

    let details = format!("{model} \u{2022} {temp_str} \u{2192} {tgt_str}");

    let mut state_parts = Vec::new();
    if heater_on {
        state_parts.push("Heater ON".to_string());
    } else {
        state_parts.push("Idle".to_string());
    }
    if pump_on {
        state_parts.push("Pump ON".to_string());
    }
    if let Some(pct) = battery {
        let icon = if charging { "\u{1f50c}" } else { "\u{1faab}" };
        state_parts.push(format!("{icon} {pct}%"));
    }
    let state = state_parts.join(" \u{2022} ");

    let assets = activity::Assets::new()
        .large_image("discord-large")
        .large_text("fumar")
        .small_image("discord-small")
        .small_text(if heater_on { "Heater ON" } else { "Idle" });

    let activity = activity::Activity::new()
        .details(&details)
        .state(&state)
        .assets(assets);

    if let Err(e) = client.set_activity(activity) {
        warn!("Discord RPC set_activity failed: {e}");
    }
}

pub fn clear() {
    let mut guard = CLIENT.lock().unwrap();
    if let Some(client) = guard.as_mut() {
        let _ = client.clear_activity();
        let _ = client.close();
    }
    *guard = None;
}
