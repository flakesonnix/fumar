use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Gauge, Paragraph},
    Frame,
};

use crate::tui::app::App;

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

pub fn draw(f: &mut Frame, app: &App) {
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
    let horizontal = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let [left, right] = horizontal.areas(area);

    draw_temp_column(f, app, left, true);
    draw_temp_column(f, app, right, false);
}

fn draw_temp_column(f: &mut Frame, app: &App, area: Rect, is_current: bool) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(rgb(42, 42, 42)));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]);
    let [label_area, value_area] = vertical.areas(inner);

    if is_current {
        let label = Paragraph::new("CURRENT")
            .style(Style::default().fg(rgb(85, 85, 85)))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(label, label_area);

        let temp = app.state.current_temp;
        let target = app.state.target_temp;
        let color = match (temp, target) {
            (Some(cur), Some(tgt)) if (cur - tgt).abs() <= 2.0 => rgb(87, 201, 122),
            (Some(_), Some(tgt)) if app.state.heater_on => {
                if app.state.current_temp.unwrap_or(0.0) < tgt {
                    rgb(232, 148, 58)
                } else {
                    rgb(94, 155, 222)
                }
            }
            (Some(_), _) if app.state.heater_on => rgb(232, 148, 58),
            _ => rgb(94, 155, 222),
        };
        let text = temp
            .map(|t| format!("{t:.1}\u{b0}C"))
            .unwrap_or_else(|| "---".into());
        let value = Paragraph::new(text)
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(value, value_area);
    } else {
        let label = Paragraph::new("TARGET")
            .style(Style::default().fg(rgb(85, 85, 85)))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(label, label_area);

        let text = app
            .state
            .target_temp
            .map(|t| format!("{t:.1}\u{b0}C"))
            .unwrap_or_else(|| "---".into());
        let value = Paragraph::new(text)
            .style(
                Style::default()
                    .fg(rgb(245, 245, 245))
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(value, value_area);
    }
}

fn draw_gauge(f: &mut Frame, app: &App, area: Rect) {
    let ratio = match (app.state.current_temp, app.state.target_temp) {
        (Some(cur), Some(tgt)) if tgt > 0.0 => (cur / tgt).clamp(0.0, 1.0),
        _ => 0.0,
    };

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(rgb(232, 148, 58)).bg(rgb(42, 42, 42)))
        .ratio(ratio as f64)
        .label(format!("{:.0}%", ratio * 100.0));
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
    let heater_para = Paragraph::new(heater_line);
    f.render_widget(heater_para, left);

    let pump_line = if app.is_crafty() {
        Line::from(vec![
            Span::styled("Pump  ", Style::default().fg(rgb(245, 245, 245))),
            Span::styled("\u{2014} N/A", Style::default().fg(rgb(85, 85, 85))),
        ])
    } else {
        let pump_indicator = if app.state.pump_on {
            Span::styled("\u{25cf}", Style::default().fg(rgb(56, 201, 201)))
        } else {
            Span::styled("\u{25cb}", Style::default().fg(rgb(85, 85, 85)))
        };
        let pump_status = if app.state.pump_on {
            Span::styled(" ON", Style::default().fg(rgb(245, 245, 245)))
        } else {
            Span::styled(" OFF", Style::default().fg(rgb(85, 85, 85)))
        };
        Line::from(vec![
            Span::styled("Pump  ", Style::default().fg(rgb(245, 245, 245))),
            pump_indicator,
            pump_status,
        ])
    };
    let pump_para = Paragraph::new(pump_line);
    f.render_widget(pump_para, right);
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
            "  \u{2191}\u{2193} temp   H heater   P pump   R refresh   Q quit",
            Style::default().fg(rgb(136, 136, 136)),
        ))
    };

    let para = Paragraph::new(line).alignment(ratatui::layout::Alignment::Left);
    f.render_widget(para, inner);
}
