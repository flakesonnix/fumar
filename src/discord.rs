use std::sync::Mutex;

use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
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

pub fn update(model: &str, temp: Option<f32>, target: Option<f32>, heater_on: bool, pump_on: bool) {
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

    let state = if pump_on {
        "Heater ON \u{2022} Pump ON".to_string()
    } else if heater_on {
        "Heater ON".to_string()
    } else {
        "Idle".to_string()
    };

    let assets = activity::Assets::new()
        .large_image("fumar-large")
        .large_text("fumar")
        .small_image("fumar-small")
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
