pub const CSS: &str = r#"
window {
    background-color: #0d0d0d;
}

.title-bar {
    background-color: #1a1a1a;
    padding: 8px 16px;
}

.title-label {
    color: #f5f5f5;
    font-weight: bold;
    font-size: 16px;
}

.model-label {
    color: #f5f5f5;
    font-size: 14px;
}

.dot-label {
    color: #888888;
    font-size: 14px;
}

.temp-section {
    background-color: #1a1a1a;
    padding: 12px;
    margin: 8px;
    border-radius: 8px;
}

.temp-label {
    color: #888888;
    font-size: 11px;
    font-weight: bold;
}

.temp-value {
    font-size: 32px;
    font-weight: bold;
    color: #57c97a;
}

.temp-value.amber {
    color: #e8943a;
}

.temp-value.blue {
    color: #5e9bde;
}

.temp-value.green {
    color: #57c97a;
}

.temp-value.gray {
    color: #555555;
}

.temp-value.white {
    color: #f5f5f5;
}

.slider-section {
    padding: 8px 16px;
    margin: 0 8px;
}

.slider-label {
    color: #f5f5f5;
    font-size: 14px;
    font-weight: bold;
}

button.heater-on {
    background-color: #e84040;
    color: white;
    font-weight: bold;
    padding: 10px 24px;
    border-radius: 8px;
    font-size: 14px;
}

button.heater-off {
    background-color: #333333;
    color: #aaaaaa;
    font-weight: bold;
    padding: 10px 24px;
    border-radius: 8px;
    font-size: 14px;
}

button.pump-on {
    background-color: #38c9c9;
    color: white;
    font-weight: bold;
    padding: 10px 24px;
    border-radius: 8px;
    font-size: 14px;
}

button.pump-off {
    background-color: #333333;
    color: #aaaaaa;
    font-weight: bold;
    padding: 10px 24px;
    border-radius: 8px;
    font-size: 14px;
}

button.pump-na {
    background-color: #222222;
    color: #555555;
    font-weight: bold;
    padding: 10px 24px;
    border-radius: 8px;
    font-size: 14px;
}

.settings-section {
    background-color: #1a1a1a;
    padding: 8px 16px;
    margin: 8px;
    border-radius: 8px;
}

.settings-label {
    color: #888888;
    font-size: 12px;
}

.error-label {
    color: #e84040;
    font-weight: bold;
    font-size: 12px;
    padding: 4px 16px;
}

.scan-label {
    color: #f5f5f5;
    font-size: 18px;
    font-weight: bold;
}

.scan-status {
    color: #888888;
    font-size: 13px;
}
"#;
