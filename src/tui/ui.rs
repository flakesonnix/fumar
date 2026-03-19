use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Gauge, Paragraph},
};

use crate::tui::app::App;

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

pub fn draw(f: &mut Frame, app: &App) {
    if app.show_settings {
        draw_settings(f, app);
        return;
    }

    let outer = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(rgb(42, 42, 42)))
        .style(Style::default().bg(rgb(13, 13, 13)));

    let area = outer.inner(f.size());
    f.render_widget(outer, f.size());

    let vertical = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(7),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
    ]);
    let [title_area, temp_area, gauge_area, status_area, help_area] = vertical.areas(area);

    draw_title(f, app, title_area);
    draw_temps(f, app, temp_area);
    draw_gauge(f, app, gauge_area);
    draw_status(f, app, status_area);
    draw_help(f, app, help_area);
}

fn draw_title(f: &mut Frame, app: &App, area: Rect) {
    let model = app.device.device_model().to_string();
    let dots = match app.tick % 3 {
        0 => "\u{b7}",
        1 => "\u{b7}\u{b7}",
        _ => "\u{b7}\u{b7}\u{b7}",
    };

    let line = Line::from(vec![
        Span::styled(
            " fumar ",
            Style::default()
                .fg(rgb(245, 245, 245))
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(model, Style::default().fg(rgb(245, 245, 245))),
        Span::raw(" "),
        Span::styled(dots, Style::default().fg(rgb(136, 136, 136))),
    ]);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(rgb(42, 42, 42)))
        .title(line);
    f.render_widget(block, area);
}

fn draw_temps(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(rgb(42, 42, 42)));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let horizontal = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let [left, right] = horizontal.areas(inner);

    // Left column: CURRENT
    let left_layout = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]);
    let [left_label, left_value] = left_layout.areas(left);

    f.render_widget(
        Paragraph::new("CURRENT")
            .style(Style::default().fg(rgb(85, 85, 85)))
            .alignment(ratatui::layout::Alignment::Center),
        left_label,
    );

    let current_color =
        if let (Some(cur), Some(tgt)) = (app.state.current_temp, app.state.target_temp) {
            if (cur - tgt).abs() <= 2.0 {
                rgb(87, 201, 122)
            } else if app.state.heater_on && cur < tgt {
                rgb(232, 148, 58)
            } else {
                rgb(94, 155, 222)
            }
        } else {
            rgb(85, 85, 85)
        };

    let current_text = app
        .state
        .current_temp
        .map(|t| format!("{t:.1}\u{b0}C"))
        .unwrap_or_else(|| "---".into());

    f.render_widget(
        Paragraph::new(current_text)
            .style(
                Style::default()
                    .fg(current_color)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(ratatui::layout::Alignment::Center),
        left_value,
    );

    // Right column: TARGET
    let right_layout = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]);
    let [right_label, right_value] = right_layout.areas(right);

    f.render_widget(
        Paragraph::new("TARGET")
            .style(Style::default().fg(rgb(85, 85, 85)))
            .alignment(ratatui::layout::Alignment::Center),
        right_label,
    );

    let target_text = app
        .state
        .target_temp
        .map(|t| format!("{t:.1}\u{b0}C"))
        .unwrap_or_else(|| "---".into());

    f.render_widget(
        Paragraph::new(target_text)
            .style(
                Style::default()
                    .fg(rgb(245, 245, 245))
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(ratatui::layout::Alignment::Center),
        right_value,
    );
}

fn draw_gauge(f: &mut Frame, app: &App, area: Rect) {
    let ratio = match (app.state.current_temp, app.state.target_temp) {
        (Some(cur), Some(tgt)) if tgt > 0.0 => (cur / tgt).clamp(0.0, 1.0) as f64,
        _ => 0.0,
    };

    let pct = (ratio * 100.0) as u16;

    let gauge = Gauge::default()
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(rgb(42, 42, 42))),
        )
        .gauge_style(Style::default().fg(rgb(232, 148, 58)).bg(rgb(42, 42, 42)))
        .ratio(ratio)
        .label(format!("{pct}%"));

    f.render_widget(gauge, area);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let horizontal = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let [left, right] = horizontal.areas(area);

    let heater_indicator = if app.state.heater_on {
        Span::styled("\u{25cf}", Style::default().fg(rgb(232, 64, 64)))
    } else {
        Span::styled("\u{25cb}", Style::default().fg(rgb(85, 85, 85)))
    };
    let heater_status = if app.state.heater_on {
        Span::styled(" ON", Style::default().fg(rgb(245, 245, 245)))
    } else {
        Span::styled(" OFF", Style::default().fg(rgb(85, 85, 85)))
    };
    let heater_line = Line::from(vec![
        Span::styled("Heater  ", Style::default().fg(rgb(245, 245, 245))),
        heater_indicator,
        heater_status,
    ]);
    f.render_widget(Paragraph::new(heater_line), left);

    if app.is_crafty() {
        let line = Line::from(vec![
            Span::styled("Pump  ", Style::default().fg(rgb(245, 245, 245))),
            Span::styled("\u{2014} N/A", Style::default().fg(rgb(85, 85, 85))),
        ]);
        f.render_widget(Paragraph::new(line), right);
    } else {
        let indicator = if app.state.pump_on {
            Span::styled("\u{25cf}", Style::default().fg(rgb(56, 201, 201)))
        } else {
            Span::styled("\u{25cb}", Style::default().fg(rgb(85, 85, 85)))
        };
        let status = if app.state.pump_on {
            Span::styled(" ON", Style::default().fg(rgb(245, 245, 245)))
        } else {
            Span::styled(" OFF", Style::default().fg(rgb(85, 85, 85)))
        };
        let line = Line::from(vec![
            Span::styled("Pump  ", Style::default().fg(rgb(245, 245, 245))),
            indicator,
            status,
        ]);
        f.render_widget(Paragraph::new(line), right);
    }
}

fn draw_help(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(rgb(42, 42, 42)));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let line = if let Some(ref msg) = app.last_error {
        Line::from(Span::styled(
            format!("  \u{26a0}  {msg}"),
            Style::default()
                .fg(rgb(232, 64, 64))
                .add_modifier(Modifier::BOLD),
        ))
    } else {
        Line::from(Span::styled(
            "  \u{2191}\u{2193} temp   H heater   P pump   R refresh   S settings   Q quit",
            Style::default().fg(rgb(136, 136, 136)),
        ))
    };

    f.render_widget(
        Paragraph::new(line).alignment(ratatui::layout::Alignment::Left),
        inner,
    );
}

fn draw_settings(f: &mut Frame, app: &App) {
    let model = app.device.device_model().to_string();
    let outer = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(rgb(42, 42, 42)))
        .style(Style::default().bg(rgb(13, 13, 13)))
        .title(Line::from(vec![
            Span::styled(
                " settings ",
                Style::default()
                    .fg(rgb(245, 245, 245))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(model, Style::default().fg(rgb(245, 245, 245))),
        ]));

    let area = outer.inner(f.size());
    f.render_widget(outer, f.size());

    let mut lines: Vec<Line> = Vec::new();

    if let Some(ref s) = app.state.settings {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  Battery:    ", Style::default().fg(rgb(245, 245, 245))),
            Span::styled(
                format!(
                    "{}%{}",
                    s.battery_level.unwrap_or(0),
                    if s.is_charging { " (charging)" } else { "" }
                ),
                Style::default().fg(rgb(136, 136, 136)),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  Unit:       ", Style::default().fg(rgb(245, 245, 245))),
            Span::styled(
                if s.is_celsius {
                    "Celsius"
                } else {
                    "Fahrenheit"
                },
                Style::default().fg(rgb(56, 201, 201)),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Auto-off:   ", Style::default().fg(rgb(245, 245, 245))),
            Span::styled(
                s.auto_shutdown_seconds
                    .map(|t| format!("{t}s"))
                    .unwrap_or_else(|| "unknown".into()),
                Style::default().fg(rgb(136, 136, 136)),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Boost viz:  ", Style::default().fg(rgb(245, 245, 245))),
            Span::styled(
                if s.boost_visualization { "ON" } else { "OFF" },
                Style::default().fg(rgb(136, 136, 136)),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Perm BT:    ", Style::default().fg(rgb(245, 245, 245))),
            Span::styled(
                if s.permanent_bluetooth { "ON" } else { "OFF" },
                Style::default().fg(rgb(136, 136, 136)),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Chrg opt:   ", Style::default().fg(rgb(245, 245, 245))),
            Span::styled(
                if s.charge_current_optimization {
                    "ON"
                } else {
                    "OFF"
                },
                Style::default().fg(rgb(136, 136, 136)),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Volt limit: ", Style::default().fg(rgb(245, 245, 245))),
            Span::styled(
                if s.charge_voltage_limit { "ON" } else { "OFF" },
                Style::default().fg(rgb(136, 136, 136)),
            ),
        ]));
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  No settings available",
            Style::default().fg(rgb(136, 136, 136)),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Press S or Esc to go back",
        Style::default().fg(rgb(85, 85, 85)),
    )));

    let para = Paragraph::new(lines);
    f.render_widget(para, area);
}
