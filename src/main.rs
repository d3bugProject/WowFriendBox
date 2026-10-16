mod config;
mod input;
mod window;

use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, TranslateMessage,
};

use crate::config::Config;
use crate::input::{AppEvent, InputManager, ACTIVE_HWND, WOW_HWNDS};
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
}

fn main() {
    print_banner();

    // 1. Load configuration
    let config = Config::load_or_create("config.json");
    println!("[CONFIG] Settings loaded from config.json");
    println!("  - Screen Resolution       : {}x{}", config.layout.screen_resolution[0], config.layout.screen_resolution[1]);
    println!("  - Center Window Size      : {}x{}", config.layout.center_window_size[0], config.layout.center_window_size[1]);
    println!("  - Background 4 Grid Slots : 1280x540 each (Top-Left, Bottom-Left, Top-Right, Bottom-Right)");
    println!("  - Prevent Background Focus: {}", config.layout.prevent_background_focus);

    // 2. Setup channel for event communication
    let (event_tx, event_rx) = mpsc::channel::<AppEvent>();

    // 3. Initialize low-level hooks on main thread
    let mut input_mgr = InputManager::new();
    input_mgr.setup(&config, event_tx);

    // 4. Controller thread: runs continuous window monitoring and handles hotkey events
    let layout_config = config.layout.clone();
    let _controller_handle = thread::spawn(move || {
        let mut wm = WindowManager::new();
        println!("\n[MONITOR] Continuous scanner active: checking for WoW windows every 300ms...");

        loop {
            // Wait up to 300ms for incoming events (F1..F5 swap, Home re-align)
            match event_rx.recv_timeout(Duration::from_millis(300)) {
                Ok(AppEvent::Swap(slot)) => {
                    println!("\n[SWAP] F{} -> Swapping [{}] to Center!", slot, slot_description(slot));
                    wm.swap_to_slot(slot, &layout_config);
                    update_shared_state(&wm);
                }
                Ok(AppEvent::ReapplyLayout) => {
                    println!("\n[LAYOUT] Re-applying layout grid manually...");
                    wm.refresh_clients(&layout_config);
                    wm.apply_layout(&layout_config);
                    update_shared_state(&wm);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Periodic scan: runs every 300ms continuously!
                    if wm.refresh_clients(&layout_config) {
                        let count = wm.clients.len();
                        println!("\n[CLIENTS] Change detected! Now tracking {} WoW client(s):", count);
                        for c in &wm.clients {
                            let role = if c.slot == wm.active_slot { "Center (Main)" } else { slot_description(c.slot) };
                            println!("   -> Slot {}: {} (PID: {}) [{}]", c.slot, c.title, c.pid, role);
                        }

                        if count > 0 {
                            println!("[LAYOUT] Positioning windows automatically...");
                            wm.apply_layout(&layout_config);
                        }
                        update_shared_state(&wm);
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    println!("[STATUS] Ready! Launch or close your Warmane accounts at any time.");
    println!("[STATUS] Keyboard broadcasting starts as: {}",
        if config.broadcast.enabled_by_default { "ON" } else { "OFF (Press Scroll Lock to toggle)" }
    );

    // 5. Win32 message pump on main thread (keeps hooks alive and responsive)
    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
