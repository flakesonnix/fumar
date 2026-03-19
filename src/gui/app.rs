use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;

use futures::channel::mpsc;
use gtk4::prelude::*;
use gtk4::{
    self as gtk, Align, Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, Label,
    Orientation, Overlay, STYLE_PROVIDER_PRIORITY_APPLICATION, Scale, Spinner,
};

use storz_rs::{DeviceModel, DeviceState};

use super::style;

#[derive(Clone)]
pub enum BleCommand {
    SetTemp(f32),
    HeaterOn,
    HeaterOff,
    PumpOn,
    PumpOff,
}

#[allow(dead_code)]
pub struct GuiApp {
    window: ApplicationWindow,
    overlay: Overlay,
    scan_box: GtkBox,
    main_box: GtkBox,
    current_temp_label: Label,
    target_temp_label: Label,
    temp_slider: Scale,
    heater_button: Button,
    pump_button: Option<Button>,
    _handler_heater_on: Rc<Cell<bool>>,
    _handler_pump_on: Rc<Cell<bool>>,
    model_label: Label,
    status_label: Label,
    error_label: Label,
    command_tx: mpsc::UnboundedSender<BleCommand>,
    pub state_rx: mpsc::UnboundedReceiver<Option<DeviceState>>,
    pub state: DeviceState,
    pub model: DeviceModel,
    pub connected: bool,
    pub tick: u64,
    error_clear_at: Option<Instant>,
}

impl GuiApp {
    pub fn new(
        app: &Application,
        model: DeviceModel,
        command_tx: mpsc::UnboundedSender<BleCommand>,
        state_rx: mpsc::UnboundedReceiver<Option<DeviceState>>,
    ) -> Rc<RefCell<Self>> {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("fumar")
            .default_width(400)
            .default_height(520)
            .resizable(true)
            .build();

        let css = CssProvider::new();
        css.load_from_data(style::CSS);

        gtk::style_context_add_provider_for_display(
            &gtk4::prelude::RootExt::display(&window),
            &css,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let overlay = Overlay::new();

        // --- Scan overlay ---
        let scan_box = GtkBox::new(Orientation::Vertical, 16);
        scan_box.set_halign(Align::Center);
        scan_box.set_valign(Align::Center);
        scan_box.set_hexpand(true);
        scan_box.set_vexpand(true);

        let spinner = Spinner::new();
        spinner.set_size_request(48, 48);
        spinner.start();
        scan_box.append(&spinner);

        let scan_label = Label::new(Some("Scanning for devices..."));
        scan_label.add_css_class("scan-label");
        scan_box.append(&scan_label);

        overlay.add_overlay(&scan_box);

        // --- Main UI ---
        let main_box = GtkBox::new(Orientation::Vertical, 0);
        main_box.set_visible(false);
        overlay.set_child(Some(&main_box));

        // Title bar
        let title_bar = GtkBox::new(Orientation::Horizontal, 8);
        title_bar.add_css_class("title-bar");

        let title = Label::new(Some(" fumar "));
        title.add_css_class("title-label");
        title.set_halign(Align::Start);
        title.set_hexpand(true);
        title_bar.append(&title);

        let model_label = Label::new(Some(&format!("{model}")));
        model_label.add_css_class("model-label");
        title_bar.append(&model_label);

        let dot_label = Label::new(Some(" \u{b7}"));
        dot_label.add_css_class("dot-label");
        title_bar.append(&dot_label);

        main_box.append(&title_bar);

        // Temperature section
        let temp_section = GtkBox::new(Orientation::Horizontal, 0);
        temp_section.add_css_class("temp-section");
        temp_section.set_homogeneous(true);

        let current_box = GtkBox::new(Orientation::Vertical, 4);
        current_box.set_halign(Align::Center);
        let current_label_text = Label::new(Some("CURRENT"));
        current_label_text.add_css_class("temp-label");
        current_box.append(&current_label_text);
        let current_temp_label = Label::new(Some("---"));
        current_temp_label.add_css_class("temp-value");
        current_temp_label.add_css_class("blue");
        current_box.append(&current_temp_label);
        temp_section.append(&current_box);

        let target_box = GtkBox::new(Orientation::Vertical, 4);
        target_box.set_halign(Align::Center);
        let target_label_text = Label::new(Some("TARGET"));
        target_label_text.add_css_class("temp-label");
        target_box.append(&target_label_text);
        let target_temp_label = Label::new(Some("---"));
        target_temp_label.add_css_class("temp-value");
        target_temp_label.add_css_class("gray");
        target_box.append(&target_temp_label);
        temp_section.append(&target_box);

        main_box.append(&temp_section);

        // Slider
        let slider_section = GtkBox::new(Orientation::Vertical, 4);
        slider_section.add_css_class("slider-section");

        let slider_text = Label::new(Some("Target Temperature"));
        slider_text.add_css_class("temp-label");
        slider_text.set_halign(Align::Start);
        slider_section.append(&slider_text);

        let temp_slider = Scale::with_range(Orientation::Horizontal, 40.0, 230.0, 1.0);
        temp_slider.set_draw_value(true);
        temp_slider.set_value_pos(gtk::PositionType::Right);
        temp_slider.set_digits(0);
        temp_slider.set_hexpand(true);
        slider_section.append(&temp_slider);

        main_box.append(&slider_section);

        // Buttons
        let button_box = GtkBox::new(Orientation::Horizontal, 12);
        button_box.set_halign(Align::Center);
        button_box.set_margin_top(8);
        button_box.set_margin_bottom(8);

        let heater_button = Button::with_label("Heater \u{25cb} OFF");
        heater_button.add_css_class("heater-off");
        button_box.append(&heater_button);

        let pump_button = if model == DeviceModel::VolcanoHybrid {
            let btn = Button::with_label("Pump  \u{25cb} OFF");
            btn.add_css_class("pump-off");
            button_box.append(&btn);
            Some(btn)
        } else {
            None
        };

        main_box.append(&button_box);

        // Settings
        let settings_section = GtkBox::new(Orientation::Vertical, 4);
        settings_section.add_css_class("settings-section");
        let status_label = Label::new(Some(&format!("{model}")));
        status_label.add_css_class("settings-label");
        settings_section.append(&status_label);
        main_box.append(&settings_section);

        // Error
        let error_label = Label::new(None);
        error_label.add_css_class("error-label");
        error_label.set_visible(false);
        main_box.append(&error_label);

        window.set_child(Some(&overlay));
        let window_clone = window.clone();

        // Shared state for signal handlers (avoid RefCell borrow in handlers)
        let handler_cmd_tx = command_tx.clone();
        let handler_heater_on: Rc<Cell<bool>> = Rc::new(Cell::new(false));
        let handler_pump_on: Rc<Cell<bool>> = Rc::new(Cell::new(false));

        // --- Build Rc ---
        let gui = Rc::new(RefCell::new(Self {
            window: window_clone,
            overlay,
            scan_box,
            main_box,
            current_temp_label,
            target_temp_label,
            temp_slider: temp_slider.clone(),
            heater_button: heater_button.clone(),
            pump_button: pump_button.clone(),
            _handler_heater_on: handler_heater_on.clone(),
            _handler_pump_on: handler_pump_on.clone(),
            model_label,
            status_label,
            error_label,
            command_tx,
            state_rx,
            state: DeviceState::default(),
            model,
            connected: false,
            tick: 0,
            error_clear_at: None,
        }));

        // --- Signal handlers ---
        {
            let cmd_tx = handler_cmd_tx.clone();
            temp_slider.connect_value_changed(move |s| {
                let _ = cmd_tx.unbounded_send(BleCommand::SetTemp(s.value() as f32));
            });
        }
        {
            let cmd_tx = handler_cmd_tx.clone();
            let h = handler_heater_on.clone();
            heater_button.connect_clicked(move |_| {
                let on = h.get();
                let _ = cmd_tx.unbounded_send(if on {
                    BleCommand::HeaterOff
                } else {
                    BleCommand::HeaterOn
                });
            });
        }
        if let Some(ref btn) = pump_button {
            let cmd_tx = handler_cmd_tx.clone();
            let p = handler_pump_on.clone();
            btn.connect_clicked(move |_| {
                let on = p.get();
                let _ = cmd_tx.unbounded_send(if on {
                    BleCommand::PumpOff
                } else {
                    BleCommand::PumpOn
                });
            });
        }

        window.present();
        gui
    }

    /// Called when BLE connects. Hide scan overlay, show main UI.
    pub fn on_connected(&mut self, initial_state: Option<DeviceState>) {
        if let Some(state) = initial_state {
            self.state = state;
        }
        self.model_label.set_text(&format!("{}", self.model));
        self.scan_box.set_visible(false);
        self.main_box.set_visible(true);
        self.update_ui();
    }

    /// Called when device disconnects.
    pub fn on_disconnected(&mut self) {
        self.error_label.set_text("Device disconnected");
        self.error_label.set_visible(true);
    }

    pub fn update_ui(&mut self) {
        let state = &self.state;

        // Current temp
        self.current_temp_label
            .set_text(&format_temp(state.current_temp));
        for c in ["green", "amber", "blue", "gray", "white"] {
            self.current_temp_label.remove_css_class(c);
        }
        self.current_temp_label.add_css_class(temp_color_class(
            state.current_temp,
            state.target_temp,
            state.heater_on,
        ));

        // Target temp
        self.target_temp_label
            .set_text(&format_temp(state.target_temp));
        self.target_temp_label.remove_css_class("gray");
        self.target_temp_label.remove_css_class("white");
        self.target_temp_label
            .add_css_class(if state.target_temp.is_some() {
                "white"
            } else {
                "gray"
            });

        // Slider
        if let Some(tgt) = state.target_temp {
            self.temp_slider.set_value(tgt as f64);
        }

        // Heater
        self._handler_heater_on.set(state.heater_on);
        if state.heater_on {
            self.heater_button.set_label("Heater \u{25cf} ON");
            self.heater_button.remove_css_class("heater-off");
            self.heater_button.add_css_class("heater-on");
        } else {
            self.heater_button.set_label("Heater \u{25cb} OFF");
            self.heater_button.remove_css_class("heater-on");
            self.heater_button.add_css_class("heater-off");
        }

        // Pump
        self._handler_pump_on.set(state.pump_on);
        if let Some(ref btn) = self.pump_button {
            if state.pump_on {
                btn.set_label("Pump  \u{25cf} ON");
                btn.remove_css_class("pump-off");
                btn.add_css_class("pump-on");
            } else {
                btn.set_label("Pump  \u{25cb} OFF");
                btn.remove_css_class("pump-on");
                btn.add_css_class("pump-off");
            }
        }

        // Status dots
        let dots = match self.tick % 3 {
            0 => "\u{b7}",
            1 => "\u{b7}\u{b7}",
            _ => "\u{b7}\u{b7}\u{b7}",
        };
        self.status_label
            .set_text(&format!("{}  {}", self.model, dots));

        // Error timeout
        if let Some(clear_at) = self.error_clear_at
            && Instant::now() >= clear_at
        {
            self.error_label.set_visible(false);
            self.error_clear_at = None;
        }
    }
}

fn format_temp(temp: Option<f32>) -> String {
    match temp {
        Some(t) => format!("{t:.1}\u{b0}C"),
        None => "---".into(),
    }
}

fn temp_color_class(current: Option<f32>, target: Option<f32>, heater_on: bool) -> &'static str {
    match (current, target) {
        (Some(cur), Some(tgt)) if (cur - tgt).abs() <= 2.0 => "green",
        (Some(cur), Some(tgt)) if heater_on && cur < tgt => "amber",
        (Some(_), Some(_)) => "blue",
        _ => "gray",
    }
}
