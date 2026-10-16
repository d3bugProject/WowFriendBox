use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub hotkeys: HotkeyConfig,
    pub broadcast: BroadcastConfig,
    pub layout: LayoutConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// Key to toggle keyboard broadcasting on/off (e.g. "ScrollLock", "F12", "Tilde")
    pub toggle_broadcast: String,
    /// Key to toggle mouse cursor broadcasting on/off (e.g. "Pause")
    pub toggle_mouse_broadcast: String,
    /// Key to re-align and reapply the window layout (e.g. "Home")
    pub reapply_layout: String,
    /// Keys to swap active foreground window to Slot 1..5
    pub slot_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastConfig {
    /// Enable key broadcasting on startup
    pub enabled_by_default: bool,
    /// Keys to broadcast to all background accounts
    pub whitelist: Vec<String>,
    /// Keys to NEVER broadcast (movements, chat, menus)
    pub blacklist: Vec<String>,
    /// Optional delay between synthetic key inputs in ms
    pub input_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// Screen resolution [width, height] (e.g. [2560, 1080])
    pub screen_resolution: [i32; 2],
    /// Foreground center window size [width, height] (e.g. [1500, 843])
    pub center_window_size: [i32; 2],
    /// Strip window borders and title bars for clean borderless mode
    pub remove_borders: bool,
    /// Block mouse clicks on background WoW windows from stealing focus
    #[serde(default = "default_true")]
    pub prevent_background_focus: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkeys: HotkeyConfig {
                toggle_broadcast: "ScrollLock".into(),
                toggle_mouse_broadcast: "Pause".into(),
                reapply_layout: "Home".into(),
                slot_keys: vec![
                    "F1".into(), "F2".into(), "F3".into(), "F4".into(), "F5".into(),
                ],
            },
            broadcast: BroadcastConfig {
                enabled_by_default: false,
                whitelist: vec![
                    "1".into(), "2".into(), "3".into(), "4".into(), "5".into(),
                    "6".into(), "7".into(), "8".into(), "9".into(), "0".into(),
                    "-".into(), "=".into(),
                    "NumPad0".into(), "NumPad1".into(), "NumPad2".into(),
                    "NumPad3".into(), "NumPad4".into(), "NumPad5".into(),
                    "NumPad6".into(), "NumPad7".into(), "NumPad8".into(),
                    "NumPad9".into(),
                ],
                blacklist: vec![
                    "Return".into(), "Space".into(), "Tab".into(), "Escape".into(),
                    "W".into(), "A".into(), "S".into(), "D".into(),
                    "Z".into(), "Q".into(),
                ],
                input_delay_ms: 0,
            },
            layout: LayoutConfig {
                screen_resolution: [2560, 1080],
                center_window_size: [1500, 843],
                remove_borders: true,
                prevent_background_focus: true,
            },
        }
    }
}

impl Config {
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> Self {
        let p = path.as_ref();
        if p.exists() {
            match fs::read_to_string(p) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(cfg) => return cfg,
                    Err(e) => {
                        eprintln!("[WARNING] Error parsing config.json: {e}. Falling back to defaults.");
                    }
                },
                Err(e) => {
                    eprintln!("[WARNING] Could not read config.json: {e}");
                }
            }
        }
        let default_cfg = Config::default();
        if let Ok(json_str) = serde_json::to_string_pretty(&default_cfg) {
            let _ = fs::write(p, json_str);
        }
        default_cfg
    }
}
