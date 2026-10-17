use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_BACK, VK_RETURN, VK_TAB};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindowThreadProcessId, IsWindowVisible,
    PostMessageW, SetForegroundWindow, WM_CHAR, WM_KEYDOWN, WM_KEYUP,
};

use crate::logger::log_event;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountEntry {
    pub slot: usize,
    pub name: String,
    pub account_name: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub character: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountsConfig {
    pub wow_path: String,
    #[serde(default = "default_delay")]
    pub launch_delay_ms: u64,
    #[serde(default = "default_true")]
    pub auto_login: bool,
    pub accounts: Vec<AccountEntry>,
}

fn default_delay() -> u64 {
    1200
}

fn default_true() -> bool {
    true
}

impl Default for AccountsConfig {
    fn default() -> Self {
        Self {
            wow_path: r"C:\Users\mnagency\Desktop\World of Warcraft 3.3.5a\Wow.exe".to_string(),
            launch_delay_ms: 1200,
            auto_login: true,
            accounts: vec![
                AccountEntry {
                    slot: 1,
                    name: "Main Tank (Center)".to_string(),
                    account_name: "COMPTE_1".to_string(),
                    password: "".to_string(),
                    character: "".to_string(),
                },
                AccountEntry {
                    slot: 2,
                    name: "Clone 1 (Top-Left)".to_string(),
                    account_name: "COMPTE_2".to_string(),
                    password: "".to_string(),
                    character: "".to_string(),
                },
                AccountEntry {
                    slot: 3,
                    name: "Clone 2 (Bottom-Left)".to_string(),
                    account_name: "COMPTE_3".to_string(),
                    password: "".to_string(),
                    character: "".to_string(),
                },
                AccountEntry {
                    slot: 4,
                    name: "Clone 3 (Top-Right)".to_string(),
                    account_name: "COMPTE_4".to_string(),
                    password: "".to_string(),
                    character: "".to_string(),
                },
                AccountEntry {
                    slot: 5,
                    name: "Clone 4 (Bottom-Right)".to_string(),
                    account_name: "COMPTE_5".to_string(),
                    password: "".to_string(),
                    character: "".to_string(),
                },
            ],
        }
    }
}

impl AccountsConfig {
    pub fn load_or_create(path_str: &str) -> Self {
        let path = Path::new(path_str);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(cfg) = serde_json::from_str::<AccountsConfig>(&content) {
                    return cfg;
                }
            }
        }

        let default_cfg = AccountsConfig::default();
        if let Ok(json) = serde_json::to_string_pretty(&default_cfg) {
            let _ = fs::write(path, json);
        }
        default_cfg
    }
}

// Thread-safe map of PID -> Account Slot
pub static LAUNCHED_PIDS: std::sync::LazyLock<RwLock<HashMap<u32, usize>>> =
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));

/// Launch all configured WoW accounts sequentially
pub fn launch_accounts(config: &AccountsConfig) {
    let path = PathBuf::from(&config.wow_path);
    if !path.exists() {
        log_event(format!(
            "[LAUNCHER ERROR] Wow.exe introuvable a l'emplacement : {}",
            config.wow_path
        ));
        return;
    }

    let work_dir = path.parent().unwrap_or(Path::new("."));

    log_event(format!(
        "[LAUNCHER] Demarrage de {} compte(s) WoW...",
        config.accounts.len()
    ));

    for acc in &config.accounts {
        log_event(format!(
            "[LAUNCHER] Lancement Slot {} ({}) -> Compte '{}'",
            acc.slot, acc.name, acc.account_name
        ));

        match Command::new(&path).current_dir(work_dir).spawn() {
            Ok(child) => {
                let pid = child.id();
                if let Ok(mut lock) = LAUNCHED_PIDS.write() {
                    lock.insert(pid, acc.slot);
                }
                log_event(format!("   -> Succes ! Processus PID {}", pid));

                if config.auto_login && !acc.account_name.is_empty() {
                    let acc_clone = acc.clone();
                    thread::spawn(move || {
                        // Wait for the window of this PID to be created
                        let mut found: Option<HWND> = None;
                        for _ in 0..50 {
                            thread::sleep(Duration::from_millis(300));
                            if let Some(h) = find_hwnd_for_pid(pid) {
                                found = Some(h);
                                break;
                            }
                        }

                        if let Some(hwnd) = found {
                            // Give WoW ~2.5 seconds to finish loading its login screen
                            thread::sleep(Duration::from_millis(2500));
                            log_event(format!(
                                "[AUTO-LOGIN] Injection des identifiants pour Slot {} ({})",
                                acc_clone.slot, acc_clone.account_name
                            ));
                            send_login_credentials(hwnd, &acc_clone.account_name, &acc_clone.password);
                        } else {
                            log_event(format!(
                                "[AUTO-LOGIN ERROR] Fenetre introuvable pour PID {}",
                                pid
                            ));
                        }
                    });
                }
            }
            Err(e) => {
                log_event(format!(
                    "   -> ERREUR lors du lancement du Slot {} : {}",
                    acc.slot, e
                ));
            }
        }

        thread::sleep(Duration::from_millis(config.launch_delay_ms));
    }

    log_event("[LAUNCHER] Tous les processus ont ete lances ! Positionnement en cours...");
}

/// Find HWND for a given Process ID
pub fn find_hwnd_for_pid(target_pid: u32) -> Option<HWND> {
    struct SearchData {
        pid: u32,
        found: Option<HWND>,
    }

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = unsafe { &mut *(lparam.0 as *mut SearchData) };
        if unsafe { IsWindowVisible(hwnd) }.as_bool() {
            let mut pid = 0u32;
            unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)); }
            if pid == data.pid {
                let mut class_buf = [0u16; 64];
                let len = unsafe { GetClassNameW(hwnd, &mut class_buf) };
                if len > 0 {
                    let class_name = String::from_utf16_lossy(&class_buf[..len as usize]);
                    if class_name.to_lowercase().contains("gxwindow") {
                        data.found = Some(hwnd);
                        return BOOL(0);
                    }
                }
            }
        }
        BOOL(1)
    }

    let mut data = SearchData {
        pid: target_pid,
        found: None,
    };

    unsafe {
        let _ = EnumWindows(Some(enum_proc), LPARAM(&mut data as *mut SearchData as isize));
    }

    data.found
}

/// Helper to send string keystrokes directly to WoW login window via WM_CHAR
pub fn send_login_credentials(hwnd: HWND, username: &str, password: &str) {
    unsafe {
        // Bring window to foreground to ensure editbox has focus
        let _ = SetForegroundWindow(hwnd);
        thread::sleep(Duration::from_millis(150));

        // Clear username box with Backspaces
        for _ in 0..25 {
            let _ = PostMessageW(Some(hwnd), WM_KEYDOWN, WPARAM(VK_BACK.0 as usize), LPARAM(0));
            thread::sleep(Duration::from_millis(10));
            let _ = PostMessageW(Some(hwnd), WM_KEYUP, WPARAM(VK_BACK.0 as usize), LPARAM(0));
            thread::sleep(Duration::from_millis(10));
        }

        // Type username via WM_CHAR
        for ch in username.chars() {
            let _ = PostMessageW(Some(hwnd), WM_CHAR, WPARAM(ch as usize), LPARAM(0));
            thread::sleep(Duration::from_millis(25));
        }

        thread::sleep(Duration::from_millis(100));

        // Tab to password box
        let _ = PostMessageW(Some(hwnd), WM_KEYDOWN, WPARAM(VK_TAB.0 as usize), LPARAM(0));
        thread::sleep(Duration::from_millis(30));
        let _ = PostMessageW(Some(hwnd), WM_KEYUP, WPARAM(VK_TAB.0 as usize), LPARAM(0));

        thread::sleep(Duration::from_millis(100));

        // Type password if provided
        if !password.is_empty() {
            for ch in password.chars() {
                let _ = PostMessageW(Some(hwnd), WM_CHAR, WPARAM(ch as usize), LPARAM(0));
                thread::sleep(Duration::from_millis(25));
            }

            thread::sleep(Duration::from_millis(120));

            // Press Enter to login
            let _ = PostMessageW(Some(hwnd), WM_KEYDOWN, WPARAM(VK_RETURN.0 as usize), LPARAM(0));
            thread::sleep(Duration::from_millis(40));
            let _ = PostMessageW(Some(hwnd), WM_KEYUP, WPARAM(VK_RETURN.0 as usize), LPARAM(0));
        }
    }
}
