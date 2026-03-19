use std::time::{Duration, Instant};

use storz_rs::{DeviceModel, DeviceState, VaporizerControl};
use tracing::warn;

pub struct App {
    pub device: Box<dyn VaporizerControl>,
    pub state: DeviceState,
    pub should_quit: bool,
    pub last_error: Option<String>,
    pub error_clear_at: Option<Instant>,
    pub tick: u64,
}

impl App {
    pub async fn new(device: Box<dyn VaporizerControl>) -> Self {
        let mut app = Self {
            device,
            state: DeviceState::default(),
            should_quit: false,
            last_error: None,
            error_clear_at: None,
            tick: 0,
        };
        app.refresh_state().await;
        app
    }

    pub async fn refresh_state(&mut self) {
        match tokio::time::timeout(Duration::from_secs(5), self.device.get_state()).await {
            Ok(Ok(state)) => {
                self.state = state;
            }
            Ok(Err(e)) => {
                self.set_error(format!("State error: {e}"));
            }
            Err(_) => {
                self.set_error("Device timeout".to_string());
            }
        }
    }

    pub async fn adjust_target(&mut self, delta: f32) {
        let new_temp = (self.state.target_temp.unwrap_or(185.0) + delta).clamp(40.0, 230.0);
        match tokio::time::timeout(
            Duration::from_secs(5),
            self.device.set_target_temperature(new_temp),
        )
        .await
        {
            Ok(Ok(_)) => {
                self.state.target_temp = Some(new_temp);
            }
            Ok(Err(e)) => self.set_error(format!("Set temp failed: {e}")),
            Err(_) => self.set_error("Timeout setting temperature".to_string()),
        }
    }

    pub async fn toggle_heater(&mut self) {
        let action = if self.state.heater_on {
            self.device.heater_off()
        } else {
            self.device.heater_on()
        };
        match tokio::time::timeout(Duration::from_secs(5), action).await {
            Ok(Ok(_)) => {
                self.state.heater_on = !self.state.heater_on;
            }
            Ok(Err(e)) => self.set_error(format!("Heater error: {e}")),
            Err(_) => self.set_error("Timeout toggling heater".to_string()),
        }
    }

    pub async fn toggle_pump(&mut self) {
        let action = if self.state.pump_on {
            self.device.pump_off()
        } else {
            self.device.pump_on()
        };
        match tokio::time::timeout(Duration::from_secs(5), action).await {
            Ok(Ok(_)) => {
                self.state.pump_on = !self.state.pump_on;
            }
            Ok(Err(e)) => {
                let msg = if e.to_string().contains("Unsupported operation") {
                    format!("Pump not supported on {}", self.device.device_model())
                } else {
                    format!("Pump error: {e}")
                };
                self.set_error(msg);
            }
            Err(_) => self.set_error("Timeout toggling pump".to_string()),
        }
    }

    pub fn set_error(&mut self, msg: String) {
        warn!("{msg}");
        self.last_error = Some(msg);
        self.error_clear_at = Some(Instant::now() + Duration::from_secs(3));
    }

    pub fn tick_errors(&mut self) {
        if let Some(clear_at) = self.error_clear_at
            && Instant::now() >= clear_at
        {
            self.last_error = None;
            self.error_clear_at = None;
        }
    }

    pub fn is_crafty(&self) -> bool {
        self.device.device_model() == DeviceModel::Crafty
    }
}
