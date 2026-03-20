use std::time::Duration;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::Terminal;

use crate::tui::app::App;
use crate::tui::ui;

pub async fn run(
    app: &mut App,
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
) -> anyhow::Result<()> {
    let mut reader = crossterm::event::EventStream::new();
    let mut interval = tokio::time::interval(Duration::from_millis(500));

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        tokio::select! {
            _ = interval.tick() => {
                app.tick += 1;
                app.tick_errors();
            }
            Some(state) = app.state_stream.next() => {
                app.apply_state(state);
                #[cfg(feature = "discord")]
                {
                    let battery = app.state.settings.as_ref().and_then(|s| s.battery_level);
                    let charging = app.state.settings.as_ref().is_some_and(|s| s.is_charging);
                    crate::discord::update(
                        &app.device.device_model().to_string(),
                        app.state.current_temp,
                        app.state.target_temp,
                        app.state.heater_on,
                        app.device.device_model() == storz_rs::DeviceModel::VolcanoHybrid && app.state.pump_on,
                        battery,
                        charging,
                    );
                }
            }
            Some(Ok(event)) = reader.next() => {
                handle_event(app, event).await;
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

async fn handle_event(app: &mut App, event: Event) {
    if let Event::Key(key) = event
        && key.kind == KeyEventKind::Press
    {
        handle_key(app, key).await;
    }
}

async fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            if app.show_settings {
                app.show_settings = false;
            } else {
                app.should_quit = true;
            }
        }
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.show_settings = !app.show_settings;
        }
        KeyCode::Up | KeyCode::Char('k') if !app.show_settings => {
            app.adjust_target(1.0).await;
        }
        KeyCode::Down | KeyCode::Char('j') if !app.show_settings => {
            app.adjust_target(-1.0).await;
        }
        KeyCode::Char('K') if !app.show_settings => {
            app.adjust_target(5.0).await;
        }
        KeyCode::Char('J') if !app.show_settings => {
            app.adjust_target(-5.0).await;
        }
        KeyCode::Char('h') | KeyCode::Char('H') if !app.show_settings => {
            app.toggle_heater().await;
        }
        KeyCode::Char('p') | KeyCode::Char('P') if !app.show_settings && app.is_volcano() => {
            app.toggle_pump().await;
        }
        KeyCode::Char('r') | KeyCode::Char('R') if !app.show_settings => {
            app.refresh_state().await;
        }
        KeyCode::Char('c') | KeyCode::Char('C') if !app.show_settings => {
            app.reconnect().await;
        }
        _ => {}
    }
}
