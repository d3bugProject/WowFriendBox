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
use std::io::Write;
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
    5000
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
/// Update resolution and fullscreen settings in WoW's Config.wtf
pub fn update_wow_config_wtf(wow_path: &str, width: i32, height: i32) -> Result<PathBuf, String> {
    let exe_path = PathBuf::from(wow_path);
    let parent = exe_path.parent().ok_or_else(|| "Chemin Wow.exe invalide".to_string())?;
    let wtf_dir = parent.join("WTF");
    if !wtf_dir.exists() {
        let _ = fs::create_dir_all(&wtf_dir);
    }
    let config_wtf_path = wtf_dir.join("Config.wtf");

    let mut lines: Vec<String> = Vec::new();
    if config_wtf_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_wtf_path) {
            lines = content.lines().map(|s| s.to_string()).collect();
        }
    }

    let res_line = format!("SET gxResolution \"{}x{}\"", width, height);
    let win_line = "SET gxWindow \"1\"".to_string();
    let max_line = "SET gxMaximize \"1\"".to_string();

    let mut has_res = false;
    let mut has_win = false;
    let mut has_max = false;

    for line in &mut lines {
        let trimmed = line.trim();
        if trimmed.to_lowercase().starts_with("set gxresolution") {
            *line = res_line.clone();
            has_res = true;
        } else if trimmed.to_lowercase().starts_with("set gxwindow") {
            *line = win_line.clone();
            has_win = true;
        } else if trimmed.to_lowercase().starts_with("set gxmaximize") {
            *line = max_line.clone();
            has_max = true;
        }
    }

    if !has_res { lines.push(res_line); }
    if !has_win { lines.push(win_line); }
    if !has_max { lines.push(max_line); }

    let new_content = lines.join("\r\n") + "\r\n";
    fs::write(&config_wtf_path, new_content).map_err(|e| format!("Erreur d'ecriture Config.wtf : {}", e))?;
    log_event(format!("[CONFIG.WTF] Resolution mise a jour vers {}x{} (Plein ecran fenetre) dans {}", width, height, config_wtf_path.display()));
    Ok(config_wtf_path)
}

/// Launch a single configured WoW account (e.g. Slot 1)
pub fn launch_single_account(config: &AccountsConfig, target_slot: usize) {
    let path = PathBuf::from(&config.wow_path);
    if !path.exists() {
        log_event(format!(
            "[LAUNCHER ERROR] Wow.exe introuvable a l'emplacement : {}",
            config.wow_path
        ));
        return;
    }

    let work_dir = path.parent().unwrap_or(Path::new("."));
    let acc = config.accounts.iter().find(|a| a.slot == target_slot).cloned().unwrap_or_else(|| {
        AccountEntry {
            slot: target_slot,
            name: format!("Slot {}", target_slot),
            account_name: "".into(),
            password: "".into(),
            character: "".into(),
        }
    });

    log_event(format!(
        "[LAUNCHER] Lancement d'un ecran unique Slot {} ({}) -> Compte '{}'",
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
                    let mut found: Option<HWND> = None;
                    for _ in 0..50 {
                        thread::sleep(Duration::from_millis(300));
                        if let Some(h) = find_hwnd_for_pid(pid) {
                            found = Some(h);
                            break;
                        }
                    }

                    if let Some(hwnd) = found {
                        thread::sleep(Duration::from_millis(2500));
                        log_event(format!(
                            "[AUTO-LOGIN] Injection des identifiants pour Slot {} ({})",
                            acc_clone.slot, acc_clone.account_name
                        ));
                        send_login_credentials(hwnd, &acc_clone.account_name, &acc_clone.password);
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
}

/// Launch all configured WoW accounts sequentially with 5-second connection delay
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
    let total_accounts = config.accounts.len();

    log_event(format!(
        "[LAUNCHER] Demarrage de {} comptes WoW avec 5s de delai entre chaque fenetre...",
        total_accounts
    ));

    for (idx, acc) in config.accounts.iter().enumerate() {
        log_event(format!(
            "[LAUNCHER] Lancement ({}/{}) Slot {} ({}) -> Compte '{}'",
            idx + 1, total_accounts, acc.slot, acc.name, acc.account_name
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
                        let mut found: Option<HWND> = None;
                        for _ in 0..50 {
                            thread::sleep(Duration::from_millis(300));
                            if let Some(h) = find_hwnd_for_pid(pid) {
                                found = Some(h);
                                break;
                            }
                        }

                        if let Some(hwnd) = found {
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

        // If there are more accounts to launch, wait 5 seconds with countdown
        if idx + 1 < total_accounts {
            let delay_secs = (config.launch_delay_ms / 1000).max(1);
            for s in (1..=delay_secs).rev() {
                print!("\r[LAUNCHER] Attente de {}s (temps de connexion au compte) avant le compte suivant... ", s);
                let _ = std::io::stdout().flush();
                thread::sleep(Duration::from_secs(1));
            }
            println!();
        }
    }

    log_event("[LAUNCHER] Tous les comptes ont ete lances ! Positionnement en cours...");
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
