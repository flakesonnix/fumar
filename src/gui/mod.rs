mod app;
mod style;

use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc;
use gtk4::prelude::*;
use gtk4::{Application, glib};
use storz_rs::{DeviceModel, DeviceState};
use tracing::{debug, info, warn};

use app::{BleCommand, GuiApp};

pub fn run_gui() {
    let app = Application::builder()
        .application_id("com.flakesonnix.fumar")
        .build();

    app.connect_activate(|app| {
        activate(app);
    });

    app.run_with_args::<&str>(&[]);
}

fn activate(app: &Application) {
    let (cmd_tx, cmd_rx) = mpsc::unbounded::<BleCommand>();
    let (state_tx, state_rx) = mpsc::unbounded::<Option<DeviceState>>();

    let gui = GuiApp::new(app, DeviceModel::Venty, cmd_tx, state_rx);

    // Poll loop: drain state channel, update UI
    let gui_clone = gui.clone();
    glib::timeout_add_local(Duration::from_millis(100), move || {
        let mut gui = gui_clone.borrow_mut();
        let mut updated = false;

        loop {
            match gui.state_rx.try_recv() {
                Ok(Some(state)) => {
                    gui.state = state;
                    updated = true;
                }
                Ok(None) => {
                    gui.on_disconnected();
                    return glib::ControlFlow::Break;
                }
                Err(_) => break,
            }
        }

        if updated && !gui.connected {
            gui.connected = true;
            let state = gui.state.clone();
            gui.on_connected(Some(state));
        } else if updated {
            gui.update_ui();
        }
        #[cfg(feature = "discord")]
        {
            if updated {
                let is_volcano = gui.model == storz_rs::DeviceModel::VolcanoHybrid;
                crate::discord::update(
                    &gui.model.to_string(),
                    gui.state.current_temp,
                    gui.state.target_temp,
                    gui.state.heater_on,
                    is_volcano && gui.state.pump_on,
                );
            }
        }

        gui.tick += 1;
        glib::ControlFlow::Continue
    });

    // Spawn BLE on background thread
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(ble_thread(cmd_rx, state_tx));
    });
}

async fn ble_thread(
    mut cmd_rx: mpsc::UnboundedReceiver<BleCommand>,
    state_tx: mpsc::UnboundedSender<Option<DeviceState>>,
) {
    let adapter = match storz_rs::get_adapter().await {
        Ok(a) => a,
        Err(e) => {
            warn!("BLE adapter error: {e}");
            let _ = state_tx.unbounded_send(None);
            return;
        }
    };

    let peripherals = match storz_rs::discover_vaporizers(&adapter, Duration::from_secs(10)).await {
        Ok(p) => p,
        Err(e) => {
            warn!("BLE scan error: {e}");
            let _ = state_tx.unbounded_send(None);
            return;
        }
    };

    if peripherals.is_empty() {
        warn!("No Storz & Bickel devices found");
        let _ = state_tx.unbounded_send(None);
        return;
    }

    let peripheral = peripherals.into_iter().next().unwrap();
    let device =
        match tokio::time::timeout(Duration::from_secs(15), storz_rs::connect(peripheral)).await {
            Ok(Ok(d)) => d,
            Ok(Err(e)) => {
                warn!("Connection failed: {e}");
                let _ = state_tx.unbounded_send(None);
                return;
            }
            Err(_) => {
                warn!("Connection timed out");
                let _ = state_tx.unbounded_send(None);
                return;
            }
        };

    info!("Connected to {}", device.device_model());

    let mut state_stream = match device.subscribe_state().await {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to subscribe to state: {e}");
            let _ = state_tx.unbounded_send(None);
            return;
        }
    };

    // Wait for first state notification
    match tokio::time::timeout(Duration::from_secs(5), state_stream.next()).await {
        Ok(Some(state)) => {
            debug!("Got initial state");
            let _ = state_tx.unbounded_send(Some(state));
        }
        _ => {
            debug!("No initial state yet");
        }
    }

    // Main loop
    loop {
        tokio::select! {
            cmd = cmd_rx.next() => {
                match cmd {
                    Some(BleCommand::SetTemp(temp)) => { let _ = device.set_target_temperature(temp).await; }
                    Some(BleCommand::HeaterOn) => { let _ = device.heater_on().await; }
                    Some(BleCommand::HeaterOff) => { let _ = device.heater_off().await; }
                    Some(BleCommand::PumpOn) => { let _ = device.pump_on().await; }
                    Some(BleCommand::PumpOff) => { let _ = device.pump_off().await; }
                    None => break,
                }
                match device.get_state().await {
                    Ok(state) => { let _ = state_tx.unbounded_send(Some(state)); }
                    Err(e) => { warn!("State read error: {e}"); }
                }
            }
            state = state_stream.next() => {
                match state {
                    Some(s) => { let _ = state_tx.unbounded_send(Some(s)); }
                    None => {
                        warn!("Device disconnected");
                        let _ = state_tx.unbounded_send(None);
                        break;
                    }
                }
            }
        }
    }
}
