use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub hotkeys: HotkeyConfig,
    pub broadcast: BroadcastConfig,
    pub layout: LayoutConfig,
    #[serde(default)]
    pub formation: FormationConfig,
    #[serde(default)]
    pub click_to_heal: ClickToHealConfig,
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
    /// Automatically suspend broadcasting when typing in chat (Enter / Slash)
    #[serde(default = "default_true")]
    pub auto_chat_pause: bool,
    /// Keys to broadcast to all background accounts
    pub whitelist: Vec<String>,
    /// Keys to NEVER broadcast (movements, chat, menus)
    pub blacklist: Vec<String>,
    /// Optional delay between synthetic key inputs in ms
    pub input_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// Layout mode: "stacked" (single fullscreen window, instant swap) or "pip"
    #[serde(default = "default_layout_mode")]
    pub mode: String,
    /// Screen resolution [width, height] (e.g. [2560, 1080])
    pub screen_resolution: [i32; 2],
    /// Foreground center window size [width, height] (e.g. [1500, 843])
    pub center_window_size: [i32; 2],
    /// Strip window borders and title bars for clean borderless mode
    pub remove_borders: bool,
    /// Block mouse clicks on background WoW windows from stealing focus
    #[serde(default = "default_false")]
    pub prevent_background_focus: bool,
}

fn default_layout_mode() -> String {
    "stacked".to_string()
}

fn default_false() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationStep {
    pub slot: usize,
    pub key: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_formation_key")]
    pub hotkey: String,
    #[serde(default = "default_formation_steps")]
    pub steps: Vec<FormationStep>,
}

fn default_formation_key() -> String {
    "X".to_string()
}

fn default_formation_steps() -> Vec<FormationStep> {
    vec![
        FormationStep { slot: 2, key: "Q".into(), duration_ms: 450 },
        FormationStep { slot: 3, key: "D".into(), duration_ms: 450 },
        FormationStep { slot: 4, key: "S".into(), duration_ms: 500 },
        FormationStep { slot: 5, key: "S".into(), duration_ms: 1200 },
    ]
}

impl Default for FormationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hotkey: default_formation_key(),
            steps: default_formation_steps(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickToHealConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_healer_slot")]
    pub healer_slot: usize,
    #[serde(default = "default_min_rx")]
    pub min_rx: f32,
    #[serde(default = "default_max_rx")]
    pub max_rx: f32,
    #[serde(default = "default_min_ry")]
    pub min_ry: f32,
    #[serde(default = "default_max_ry")]
    pub max_ry: f32,
}

fn default_healer_slot() -> usize { 5 }
fn default_min_rx() -> f32 { 0.005 }
fn default_max_rx() -> f32 { 0.20 }
fn default_min_ry() -> f32 { 0.25 }
fn default_max_ry() -> f32 { 0.75 }

impl Default for ClickToHealConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            healer_slot: default_healer_slot(),
            min_rx: default_min_rx(),
            max_rx: default_max_rx(),
            min_ry: default_min_ry(),
            max_ry: default_max_ry(),
        }
    }
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
                auto_chat_pause: true,
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
                mode: "stacked".into(),
                screen_resolution: [1920, 1080],
                center_window_size: [1920, 1080],
                remove_borders: false,
                prevent_background_focus: false,
            },
            formation: FormationConfig::default(),
            click_to_heal: ClickToHealConfig::default(),
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
