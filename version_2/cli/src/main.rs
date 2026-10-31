#![allow(dead_code)]

mod config;
mod input;
mod launcher;
mod logger;
mod window;

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, TranslateMessage,
};

use crate::config::Config;
use crate::input::{AppEvent, InputManager, ACTIVE_HWND, WOW_HWNDS};
use crate::logger::log_event;
use crate::window::{slot_description, SafeHwnd, WindowManager};

fn print_hotkey_banner() {
    println!(r#"
===================================================================
            WOWFRIENDBOX v2 - CONTROLEUR & BROADCAST ACTIF
===================================================================
  [Ctrl + 1..5] : Permuter vers la fenetre WoW Slot 1 a 5
  [F1]          : Follow (Fait suivre tous les clones au leader)
  [F2]          : Unfollow + Deploiement (Formation en eventail)
  [F3]          : Toggle Plein Ecran (Borderless sans bordure)
  [F4]          : Toggle Mode Fenetre (Fenetre standard avec bordures)
  [F5]          : Toggle Barre de fenetre (Bordures & Titre)
  [F6]          : Toggle Barre des taches Windows (Immersion totale)
  [Alt + R]     : Recharger l'interface (/reload) sur les 5 fenetres
  [F, G]        : Interaction PNJ (Broadcast actif)
  [F10]         : Arreter / Basculer l'ecoute du Broadcast (ON/OFF)
  -----------------------------------------------------------------
  Touches broadcastées : 1..0, Alt+1..0, Ctrl+1..0, Shift+1..0, Alt+A..Z
===================================================================
[CONSOLE] Le controleur surveille les fenetres. Appuyez sur Ctrl+C pour quitter.
"#);
}

fn read_user_choice() -> String {
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    input.trim().to_string()
}

fn select_resolution_prompt() -> Option<[i32; 2]> {
    println!("\n--- Choix de la resolution (Plein Ecran) ---");
    println!("  [1] 2560x1080 (Ultrawide 21:9)");
    println!("  [2] 1920x1080 (Full HD 16:9 standard)");
    println!("  [0] Annuler / Retour");
    print!("Votre choix [1/2/0] : ");
    let _ = io::stdout().flush();

    match read_user_choice().as_str() {
        "1" => Some([2560, 1080]),
        "2" => Some([1920, 1080]),
        _ => None,
    }
}

fn update_shared_state(wm: &WindowManager) {
    let hwnds: Vec<SafeHwnd> = wm.clients.iter().map(|c| c.hwnd).collect();
    let active_h = wm
        .clients
        .iter()
        .find(|c| c.slot == wm.active_slot)
        .map(|c| c.hwnd.as_isize())
        .unwrap_or(0);

    ACTIVE_HWND.store(active_h, Ordering::SeqCst);
    if let Ok(mut lock) = WOW_HWNDS.write() {
        *lock = hwnds;
    }

    if let Ok(mut lock) = crate::input::SLOT_HWNDS.write() {
        lock.clear();
        for c in &wm.clients {
            lock.insert(c.slot, c.hwnd);
        }
    }
}


fn run_controller(config: Config, _accounts_cfg: launcher::AccountsConfig) {
    print_hotkey_banner();

    // 1. Setup channel for event communication
    let (event_tx, event_rx) = mpsc::channel::<AppEvent>();

    // 2. Dedicated hook thread: installs Win32 low-level hooks and runs hook message pump
    let hook_config = config.clone();
    let hook_event_tx = event_tx.clone();
    let _hook_handle = thread::spawn(move || {
        let mut input_mgr = InputManager::new();
        input_mgr.setup(&hook_config, hook_event_tx);
        unsafe {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    });

    // 3. Controller thread: runs continuous window monitoring and handles hotkey events
    let layout_config = config.layout.clone();
    let _controller_handle = thread::spawn(move || {
        let mut wm = WindowManager::new();
        log_event("[SCANNER] Surveillance continue active (300ms)...");

        loop {
            match event_rx.recv_timeout(Duration::from_millis(300)) {
                Ok(AppEvent::Swap(slot)) => {
                    log_event(format!("[SWAP] Ctrl+{} -> Slot #{} ({}) au premier plan", slot, slot, slot_description(slot)));
                    wm.swap_to_slot(slot, &layout_config);
                    update_shared_state(&wm);
                }
                Ok(AppEvent::Follow) => {
                    wm.follow_all();
                }
                Ok(AppEvent::UnfollowFormation) => {
                    log_event("[FORMATION] Unfollow + Deploiement en eventail (F2)");
                    crate::input::execute_formation();
                }
                Ok(AppEvent::ToggleFullscreen) => {
                    wm.toggle_fullscreen(&layout_config);
                    update_shared_state(&wm);
                }
                Ok(AppEvent::ToggleWindowed) => {
                    wm.toggle_windowed(&layout_config);
                    update_shared_state(&wm);
                }
                Ok(AppEvent::ToggleBorders) => {
                    wm.toggle_borders();
                    update_shared_state(&wm);
                }
                Ok(AppEvent::ToggleTaskbar) => {
                    wm.toggle_taskbar();
                }
                Ok(AppEvent::ReloadAllUi) => {
                    wm.reload_all_ui();
                    wm.apply_layout(&layout_config);
                    update_shared_state(&wm);
                }
                Ok(AppEvent::ReapplyLayout) => {
                    log_event("[LAYOUT] Realignement de la disposition...");
                    wm.refresh_clients(&layout_config);
                    wm.apply_layout(&layout_config);
                    update_shared_state(&wm);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if wm.refresh_clients(&layout_config) {
                        let count = wm.clients.len();
                        log_event(format!("[CLIENTS] {} client(s) WoW detecte(s)", count));
                        for c in &wm.clients {
                            let role = if c.slot == wm.active_slot { "Centre (Leader)" } else { slot_description(c.slot) };
                            println!("   -> Slot {}: {} (PID: {}) [{}]", c.slot, c.title, c.pid, role);
                        }

                        if count > 0 {
                            log_event("[LAYOUT] Positionnement automatique de la disposition");
                            wm.apply_layout(&layout_config);
                        }
                        update_shared_state(&wm);
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    log_event(format!(
        "[STATUT] Clavier au demarrage : {}",
        if config.broadcast.enabled_by_default { "ON" } else { "OFF" }
    ));

    // Message pump for console / main thread
    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn main() {
    let mut config = Config::load_or_create("config.json");
    let accounts_cfg = launcher::AccountsConfig::load_or_create("accounts.json");

    loop {
        println!(r#"
===================================================================
      WOWFRIENDBOX v2 - Multiboxing Controller (Warmane)
===================================================================
  [1] Lancer 1 ecran WoW (avec selection resolution)
  [2] Lancer 5 ecrans WoW (5 comptes, delai 5s, selection resolution)
  [3] Ouvrir config.json (Configuration generale & touches)
  [4] Ouvrir Config.wtf de WoW (Resolution & parametres WoW)
  [5] Ouvrir accounts.json (Comptes & Identifiants)
  [6] Demarrer le Broadcasting & Controleur (si WoW deja lance)
  [0] Quitter
===================================================================
"#);
        print!("Votre choix [1/2/3/4/5/6/0] : ");
        let _ = io::stdout().flush();

        let choice = read_user_choice();
        match choice.as_str() {
            "1" => {
                if let Some(res) = select_resolution_prompt() {
                    let _ = launcher::update_wow_config_wtf(&accounts_cfg.wow_path, res[0], res[1]);
                    config.layout.screen_resolution = res;
                    config.layout.center_window_size = res;
                    let _ = config.save("config.json");

                    let launch_cfg = accounts_cfg.clone();
                    thread::spawn(move || {
                        thread::sleep(Duration::from_millis(500));
                        launcher::launch_single_account(&launch_cfg, 1);
                    });

                    run_controller(config, accounts_cfg);
                    break;
                }
            }
            "2" => {
                if let Some(res) = select_resolution_prompt() {
                    let _ = launcher::update_wow_config_wtf(&accounts_cfg.wow_path, res[0], res[1]);
                    config.layout.screen_resolution = res;
                    config.layout.center_window_size = res;
                    let _ = config.save("config.json");

                    let launch_cfg = accounts_cfg.clone();
                    thread::spawn(move || {
                        thread::sleep(Duration::from_millis(500));
                        launcher::launch_accounts(&launch_cfg);
                    });

                    run_controller(config, accounts_cfg);
                    break;
                }
            }
            "3" => {
                println!("[OUVERTURE] Ouverture de config.json...");
                let _ = Command::new("notepad.exe").arg("config.json").spawn();
            }
            "4" => {
                let exe_path = PathBuf::from(&accounts_cfg.wow_path);
                if let Some(parent) = exe_path.parent() {
                    let config_wtf = parent.join("WTF").join("Config.wtf");
                    if config_wtf.exists() {
                        println!("[OUVERTURE] Ouverture de Config.wtf ({:?})...", config_wtf);
                        let _ = Command::new("notepad.exe").arg(config_wtf).spawn();
                    } else {
                        println!("[AVERTISSEMENT] Config.wtf introuvable a l'emplacement : {:?}", config_wtf);
                    }
                }
            }
            "5" => {
                println!("[OUVERTURE] Ouverture de accounts.json...");
                let _ = Command::new("notepad.exe").arg("accounts.json").spawn();
            }
            "6" => {
                println!("[DEMARRAGE] Lancement direct du controleur et du broadcast...");
                run_controller(config, accounts_cfg);
                break;
            }
            "0" | "q" | "exit" => {
                println!("Fermeture de WowFriendBox.");
                break;
            }
            _ => {
                println!("Choix invalide, veuillez reessayer.");
            }
        }
    }
}
