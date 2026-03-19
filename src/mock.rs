use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::Stream;
use futures::StreamExt;
use storz_rs::{DeviceModel, DeviceState, StorzError, VaporizerControl};
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio_stream::wrappers::BroadcastStream;

/// Mock Venty emulator for testing without BLE hardware.
///
/// Simulates temperature dynamics:
/// - Heater on → current temp rises toward target (roughly +1°C per tick)
/// - Heater off → current temp cools toward 20°C
/// - Setpoint reached flag set when within 2°C of target
pub struct MockVenty {
    state: Arc<Mutex<DeviceState>>,
    state_tx: broadcast::Sender<DeviceState>,
    target: Arc<RwLock<f32>>,
    heater: Arc<RwLock<bool>>,
}

impl MockVenty {
    pub fn new() -> Self {
        let (state_tx, _) = broadcast::channel(16);
        let target = Arc::new(RwLock::new(180.0));
        let heater = Arc::new(RwLock::new(false));

        let state = Arc::new(Mutex::new(DeviceState {
            current_temp: Some(22.0),
            target_temp: Some(180.0),
            heater_on: false,
            pump_on: false,
            fan_on: false,
            setpoint_reached: false,
            raw_activity: None,
        }));

        let mock = Self {
            state: state.clone(),
            state_tx: state_tx.clone(),
            target: target.clone(),
            heater: heater.clone(),
        };

        mock.spawn_simulator(state, state_tx, target, heater);
        mock
    }

    fn spawn_simulator(
        &self,
        state: Arc<Mutex<DeviceState>>,
        state_tx: broadcast::Sender<DeviceState>,
        target: Arc<RwLock<f32>>,
        heater: Arc<RwLock<bool>>,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;

                let mut s = state.lock().await;
                let tgt = *target.read().await;
                let is_heating = *heater.read().await;

                s.target_temp = Some(tgt);
                s.heater_on = is_heating;

                if let Some(cur) = s.current_temp {
                    let new_temp = if is_heating {
                        // Rise toward target: faster when further away
                        let diff = tgt - cur;
                        let step = (diff * 0.05).clamp(0.5, 5.0);
                        (cur + step).min(tgt)
                    } else {
                        // Cool toward ambient (20°C)
                        let diff = cur - 20.0;
                        let step = (diff * 0.03).clamp(0.2, 3.0);
                        (cur - step).max(20.0)
                    };
                    s.current_temp = Some(new_temp);
                    s.setpoint_reached = (new_temp - tgt).abs() <= 2.0;
                }

                let _ = state_tx.send(s.clone());
            }
        });
    }
}

#[async_trait]
impl VaporizerControl for MockVenty {
    async fn get_current_temperature(&self) -> Result<f32, StorzError> {
        let state = self.state.lock().await;
        state.current_temp.ok_or(StorzError::ParseError(
            "Current temperature not available".into(),
        ))
    }

    async fn get_target_temperature(&self) -> Result<f32, StorzError> {
        let state = self.state.lock().await;
        state
            .target_temp
            .ok_or(StorzError::ParseError("Target temperature not available".into()))
    }

    async fn set_target_temperature(&self, celsius: f32) -> Result<(), StorzError> {
        let celsius = (celsius / 2.0).round() * 2.0;
        let celsius = celsius.clamp(40.0, 230.0);
        *self.target.write().await = celsius;
        let mut state = self.state.lock().await;
        state.target_temp = Some(celsius);
        Ok(())
    }

    async fn heater_on(&self) -> Result<(), StorzError> {
        *self.heater.write().await = true;
        let mut state = self.state.lock().await;
        state.heater_on = true;
        Ok(())
    }

    async fn heater_off(&self) -> Result<(), StorzError> {
        *self.heater.write().await = false;
        let mut state = self.state.lock().await;
        state.heater_on = false;
        Ok(())
    }

    async fn pump_on(&self) -> Result<(), StorzError> {
        Err(StorzError::UnsupportedOperation {
            device: "Venty (mock)".into(),
            operation: "pump_on".into(),
        })
    }

    async fn pump_off(&self) -> Result<(), StorzError> {
        Err(StorzError::UnsupportedOperation {
            device: "Venty (mock)".into(),
            operation: "pump_off".into(),
        })
    }

    async fn get_state(&self) -> Result<DeviceState, StorzError> {
        let state = self.state.lock().await;
        Ok(state.clone())
    }

    async fn subscribe_state(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = DeviceState> + Send>>, StorzError> {
        let rx = self.state_tx.subscribe();
        Ok(Box::pin(
            BroadcastStream::new(rx).filter_map(|r| async move { r.ok() }),
        ))
    }

    fn device_model(&self) -> DeviceModel {
        DeviceModel::Venty
    }
}
