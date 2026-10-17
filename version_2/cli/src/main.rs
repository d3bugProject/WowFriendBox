mod config;
mod gui;
mod input;
mod launcher;
mod logger;
mod window;

use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, TranslateMessage,
};

use crate::config::Config;
use crate::gui::{run_gui, GUI_SLOTS, GuiClientSlot};
use crate::input::{AppEvent, InputManager, ACTIVE_HWND, WOW_HWNDS};
use crate::logger::log_event;
use crate::window::{slot_description, SafeHwnd, WindowManager};

fn print_banner() {
    println!(r#"
===================================================================
      ISBOXER-RS - Multiboxing Controller (Warmane & WoW)
===================================================================
  [F1]          : Center Main Window (1500x843)
  [F2]          : Top-Left Window (1280x540)  -> Center
  [F3]          : Bottom-Left Window (1280x540) -> Center
  [F4]          : Top-Right Window (1280x540) -> Center
  [F5]          : Bottom-Right Window (1280x540) -> Center
  -----------------------------------------------------------------
  [Scroll Lock] : Toggle Keyboard Broadcasting (ON / OFF)
  [Pause]       : Toggle Mouse Broadcasting (ON / OFF)
  [Home]        : Re-align & Re-apply Window Layout
  [X]           : Formation / Combat Spread (Ecarter Clones & Recul Heal)
===================================================================
"#);
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

    // Synchronize client slots with GUI for real-time status and FPS watch
    if let Ok(mut lock) = GUI_SLOTS.write() {
        lock.clear();
        for c in &wm.clients {
            let is_active = c.slot == wm.active_slot;
            lock.push(GuiClientSlot {
                slot: c.slot,
                title: c.title.clone(),
                pid: c.pid,
                is_active,
                fps: if is_active { 60 } else { 30 },
            });
        }
        lock.sort_by_key(|s| s.slot);
    }
}

fn main() {
    print_banner();

    // 1. Load configuration
    let config = Config::load_or_create("config.json");
    let accounts_cfg = launcher::AccountsConfig::load_or_create("accounts.json");
    log_event("[CONFIG] Parametres charges depuis config.json et accounts.json");
    println!("  - Resolution Ecran        : {}x{}", config.layout.screen_resolution[0], config.layout.screen_resolution[1]);
    println!("  - Taille Fenetre Centre   : {}x{}", config.layout.center_window_size[0], config.layout.center_window_size[1]);
    println!("  - 4 Slots Arriere-plan    : 1280x540");
    println!("  - Anti-Vol Focus          : {}", config.layout.prevent_background_focus);
    println!("  - Chat Auto-Pause (Enter) : {}", if config.broadcast.auto_chat_pause { "ACTIF (ON)" } else { "DESACTIF (OFF)" });
    println!("  - Formation Hotkey        : [{}] (Actif: {})", config.formation.hotkey, config.formation.enabled);
    println!("  - Wow.exe configure       : {}", accounts_cfg.wow_path);

    let should_launch = std::env::args().any(|a| a == "--launch" || a == "-l");
    if should_launch {
        let launch_cfg = accounts_cfg.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(500));
            launcher::launch_accounts(&launch_cfg);
        });
    }

    // 2. Setup channel for event communication
    let (event_tx, event_rx) = mpsc::channel::<AppEvent>();

    // 3. Dedicated hook thread: installs Win32 low-level hooks and runs hook message pump
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

    // 4. Controller thread: runs continuous window monitoring and handles hotkey events
    let layout_config = config.layout.clone();
    let _controller_handle = thread::spawn(move || {
        let mut wm = WindowManager::new();
        log_event("[SCANNER] Surveillance continue active (300ms)...");

        loop {
            match event_rx.recv_timeout(Duration::from_millis(300)) {
                Ok(AppEvent::Swap(slot)) => {
                    log_event(format!("[SWAP] F{} -> [{}] au Centre", slot, slot_description(slot)));
                    wm.swap_to_slot(slot, &layout_config);
                    update_shared_state(&wm);
                }
                Ok(AppEvent::ReapplyLayout) => {
                    log_event("[LAYOUT] Realignement de la grille...");
                    wm.refresh_clients(&layout_config);
                    wm.apply_layout(&layout_config);
                    update_shared_state(&wm);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if wm.refresh_clients(&layout_config) {
                        let count = wm.clients.len();
                        log_event(format!("[CLIENTS] {} client(s) WoW detecte(s)", count));
                        for c in &wm.clients {
                            let role = if c.slot == wm.active_slot { "Centre (Main)" } else { slot_description(c.slot) };
                            println!("   -> Slot {}: {} (PID: {}) [{}]", c.slot, c.title, c.pid, role);
                        }

                        if count > 0 {
                            log_event("[LAYOUT] Positionnement automatique de la grille");
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

    let is_cli = std::env::args().any(|a| a == "--cli" || a == "-c" || a == "--no-gui");

    if is_cli {
        log_event("[MODE] Demarrage en mode CLI console pure (sans interface graphique)");
        println!("[CONSOLE] Appuyez sur Ctrl+C dans ce terminal pour quitter.");

        unsafe {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    } else {
        // 5. Run GUI on main thread (1000x100 HUD overlay with custom frame, buttons, logs, FPS watch)
        if let Err(e) = run_gui(event_tx) {
            eprintln!("[GUI ERROR] Impossible de lancer l'interface graphique : {:?}", e);
        }
    }
}
