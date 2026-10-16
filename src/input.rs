use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Mutex, RwLock};

use windows::Win32::Foundation::{LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    MapVirtualKeyW, MAPVK_VK_TO_VSC, VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_6,
    VK_7, VK_8, VK_9, VK_A, VK_D, VK_ESCAPE, VK_F1, VK_F10, VK_F11, VK_F12,
    VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_HOME, VK_NUMPAD0,
    VK_NUMPAD1, VK_NUMPAD2, VK_NUMPAD3, VK_NUMPAD4, VK_NUMPAD5, VK_NUMPAD6, VK_NUMPAD7,
    VK_NUMPAD8, VK_NUMPAD9, VK_PAUSE, VK_Q, VK_RETURN, VK_S, VK_SCROLL, VK_SPACE,
    VK_TAB, VK_W, VK_Z,
};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetAncestor, GetClientRect, GetCursorPos, GetForegroundWindow,
    PostMessageW, SetWindowsHookExW, UnhookWindowsHookEx, WindowFromPoint, GA_ROOT,
    HHOOK, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_RBUTTONDOWN,
    WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::config::Config;
use crate::window::SafeHwnd;

#[derive(Debug, Clone, Copy)]
pub enum AppEvent {
    Swap(usize),
    ReapplyLayout,
}

// Lock-free shared state for Win32 hooks
pub static BROADCAST_ENABLED: AtomicBool = AtomicBool::new(false);
pub static MOUSE_BROADCAST_ENABLED: AtomicBool = AtomicBool::new(false);
pub static PREVENT_BG_FOCUS: AtomicBool = AtomicBool::new(true);
pub static ACTIVE_HWND: AtomicIsize = AtomicIsize::new(0);

pub static WOW_HWNDS: RwLock<Vec<SafeHwnd>> = RwLock::new(Vec::new());
pub static EVENT_SENDER: Mutex<Option<Sender<AppEvent>>> = Mutex::new(None);

pub static TOGGLE_BROADCAST_VK: AtomicIsize = AtomicIsize::new(0);
pub static TOGGLE_MOUSE_VK: AtomicIsize = AtomicIsize::new(0);
pub static REAPPLY_LAYOUT_VK: AtomicIsize = AtomicIsize::new(0);

pub static SLOT_VKS: RwLock<Vec<u16>> = RwLock::new(Vec::new());
pub static WHITELIST_VKS: RwLock<Vec<u16>> = RwLock::new(Vec::new());
pub static BLACKLIST_VKS: RwLock<Vec<u16>> = RwLock::new(Vec::new());

pub struct InputManager {
    kbd_hook: HHOOK,
    mouse_hook: HHOOK,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            kbd_hook: HHOOK::default(),
            mouse_hook: HHOOK::default(),
        }
    }

    pub fn setup(&mut self, config: &Config, event_tx: Sender<AppEvent>) {
        BROADCAST_ENABLED.store(config.broadcast.enabled_by_default, Ordering::Relaxed);
        MOUSE_BROADCAST_ENABLED.store(false, Ordering::Relaxed);
        PREVENT_BG_FOCUS.store(config.layout.prevent_background_focus, Ordering::Relaxed);

        let toggle_broadcast_vk = parse_key_name(&config.hotkeys.toggle_broadcast);
        let toggle_mouse_vk = parse_key_name(&config.hotkeys.toggle_mouse_broadcast);
        let reapply_layout_vk = parse_key_name(&config.hotkeys.reapply_layout);
        let slot_vks: Vec<u16> = config.hotkeys.slot_keys.iter().map(|s| parse_key_name(s)).collect();
        let whitelist_vks: Vec<u16> = config.broadcast.whitelist.iter().map(|s| parse_key_name(s)).collect();
        let blacklist_vks: Vec<u16> = config.broadcast.blacklist.iter().map(|s| parse_key_name(s)).collect();

        TOGGLE_BROADCAST_VK.store(toggle_broadcast_vk as isize, Ordering::Relaxed);
        TOGGLE_MOUSE_VK.store(toggle_mouse_vk as isize, Ordering::Relaxed);
        REAPPLY_LAYOUT_VK.store(reapply_layout_vk as isize, Ordering::Relaxed);

        if let Ok(mut lock) = SLOT_VKS.write() { *lock = slot_vks; }
        if let Ok(mut lock) = WHITELIST_VKS.write() { *lock = whitelist_vks; }
        if let Ok(mut lock) = BLACKLIST_VKS.write() { *lock = blacklist_vks; }

        if let Ok(mut lock) = EVENT_SENDER.lock() {
            *lock = Some(event_tx);
        }

        unsafe {
            if let Ok(hook) = SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), None, 0) {
                self.kbd_hook = hook;
            }
            if let Ok(hook) = SetWindowsHookExW(WH_MOUSE_LL, Some(low_level_mouse_proc), None, 0) {
                self.mouse_hook = hook;
            }
        }
    }
}

impl Drop for InputManager {
    fn drop(&mut self) {
        unsafe {
            if !self.kbd_hook.is_invalid() {
                let _ = UnhookWindowsHookEx(self.kbd_hook);
            }
            if !self.mouse_hook.is_invalid() {
                let _ = UnhookWindowsHookEx(self.mouse_hook);
            }
        }
    }
}

/// Converts key names into Win32 Virtual Key Codes
pub fn parse_key_name(name: &str) -> u16 {
    match name.to_uppercase().as_str() {
        "1" => VK_1.0,
        "2" => VK_2.0,
        "3" => VK_3.0,
        "4" => VK_4.0,
        "5" => VK_5.0,
        "6" => VK_6.0,
        "7" => VK_7.0,
        "8" => VK_8.0,
        "9" => VK_9.0,
        "0" => VK_0.0,
        "F1" => VK_F1.0,
        "F2" => VK_F2.0,
        "F3" => VK_F3.0,
        "F4" => VK_F4.0,
        "F5" => VK_F5.0,
        "F6" => VK_F6.0,
        "F7" => VK_F7.0,
        "F8" => VK_F8.0,
        "F9" => VK_F9.0,
        "F10" => VK_F10.0,
        "F11" => VK_F11.0,
        "F12" => VK_F12.0,
        "NUMPAD0" => VK_NUMPAD0.0,
        "NUMPAD1" => VK_NUMPAD1.0,
        "NUMPAD2" => VK_NUMPAD2.0,
        "NUMPAD3" => VK_NUMPAD3.0,
        "NUMPAD4" => VK_NUMPAD4.0,
        "NUMPAD5" => VK_NUMPAD5.0,
        "NUMPAD6" => VK_NUMPAD6.0,
        "NUMPAD7" => VK_NUMPAD7.0,
        "NUMPAD8" => VK_NUMPAD8.0,
        "NUMPAD9" => VK_NUMPAD9.0,
        "SCROLLLOCK" => VK_SCROLL.0,
        "PAUSE" => VK_PAUSE.0,
        "HOME" => VK_HOME.0,
        "SPACE" => VK_SPACE.0,
        "RETURN" | "ENTER" => VK_RETURN.0,
        "TAB" => VK_TAB.0,
        "ESCAPE" | "ESC" => VK_ESCAPE.0,
        "W" => VK_W.0,
        "A" => VK_A.0,
        "S" => VK_S.0,
        "D" => VK_D.0,
        "Z" => VK_Z.0,
        "Q" => VK_Q.0,
        _ => 0,
    }
}

/// Low-level keyboard hook procedure (deadlock-free via channel)
unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let kbd_struct = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        let vk = kbd_struct.vkCode as u16;
        let is_key_down = wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize;
        let is_key_up = wparam.0 == WM_KEYUP as usize || wparam.0 == WM_SYSKEYUP as usize;

        if is_key_down {
            let toggle_b = TOGGLE_BROADCAST_VK.load(Ordering::Relaxed) as u16;
            if toggle_b != 0 && vk == toggle_b {
                let cur = BROADCAST_ENABLED.load(Ordering::Relaxed);
                BROADCAST_ENABLED.store(!cur, Ordering::Relaxed);
                println!(
                    "\n[ISBOXER-RS] Keyboard Broadcasting: {}",
                    if !cur { "ACTIVE (ON)" } else { "INACTIVE (OFF)" }
                );
                return LRESULT(1);
            }

            let toggle_m = TOGGLE_MOUSE_VK.load(Ordering::Relaxed) as u16;
            if toggle_m != 0 && vk == toggle_m {
                let cur = MOUSE_BROADCAST_ENABLED.load(Ordering::Relaxed);
                MOUSE_BROADCAST_ENABLED.store(!cur, Ordering::Relaxed);
                println!(
                    "\n[ISBOXER-RS] Mouse Broadcasting: {}",
                    if !cur { "ACTIVE (ON)" } else { "INACTIVE (OFF)" }
                );
                return LRESULT(1);
            }

            let reapply = REAPPLY_LAYOUT_VK.load(Ordering::Relaxed) as u16;
            if reapply != 0 && vk == reapply {
                if let Ok(guard) = EVENT_SENDER.lock() {
                    if let Some(sender) = guard.as_ref() {
                        let _ = sender.send(AppEvent::ReapplyLayout);
                    }
                }
                return LRESULT(1);
            }

            if let Ok(slots) = SLOT_VKS.read() {
                for (i, &slot_vk) in slots.iter().enumerate() {
                    if vk == slot_vk {
                        if let Ok(guard) = EVENT_SENDER.lock() {
                            if let Some(sender) = guard.as_ref() {
                                let _ = sender.send(AppEvent::Swap(i + 1));
                            }
                        }
                        return LRESULT(1);
                    }
                }
            }
        }

        // Keyboard broadcasting
        if BROADCAST_ENABLED.load(Ordering::Relaxed) && (is_key_down || is_key_up) {
            let fg_hwnd = unsafe { GetForegroundWindow() };
            let act = ACTIVE_HWND.load(Ordering::Relaxed);
            let is_active_wow = act != 0 && fg_hwnd.0 as isize == act;

            if is_active_wow {
                let is_whitelisted = if let Ok(wl) = WHITELIST_VKS.read() {
                    wl.is_empty() || wl.contains(&vk)
                } else {
                    true
                };
                let is_blacklisted = if let Ok(bl) = BLACKLIST_VKS.read() {
                    bl.contains(&vk)
                } else {
                    false
                };

                if is_whitelisted && !is_blacklisted {
                    let scan_code = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) };
                    let mut lp: u32 = 1 | (scan_code << 16);
                    if is_key_up {
                        lp |= (1 << 30) | (1 << 31);
                    }
                    let msg = if is_key_down { WM_KEYDOWN } else { WM_KEYUP };

                    if let Ok(hwnds) = WOW_HWNDS.read() {
                        for hwnd in hwnds.iter() {
                            if hwnd.0 != fg_hwnd {
                                unsafe {
                                    let _ = PostMessageW(
                                        Some(hwnd.0),
                                        msg,
                                        WPARAM(vk as usize),
                                        LPARAM(lp as isize),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    unsafe { CallNextHookEx(None, n_code, wparam, lparam) }
}

/// Low-level mouse hook procedure (deadlock-free)
unsafe extern "system" fn low_level_mouse_proc(
    n_code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let msg = wparam.0 as u32;
        let is_click = msg == WM_LBUTTONDOWN
            || msg == WM_LBUTTONUP
            || msg == WM_RBUTTONDOWN
            || msg == WM_RBUTTONUP
            || msg == WM_MBUTTONDOWN
            || msg == WM_MBUTTONUP;

        if is_click {
            let mut pt = POINT::default();
            unsafe { let _ = GetCursorPos(&mut pt); };

            let hwnd_under = unsafe { WindowFromPoint(pt) };
            let root_hwnd = unsafe { GetAncestor(hwnd_under, GA_ROOT) };

            let act = ACTIVE_HWND.load(Ordering::Relaxed);

            // 1. Block background clicks from stealing focus
            if PREVENT_BG_FOCUS.load(Ordering::Relaxed) {
                if let Ok(hwnds) = WOW_HWNDS.read() {
                    let is_bg_wow = hwnds.iter().any(|h| h.0 == root_hwnd) && root_hwnd.0 as isize != act;
                    if is_bg_wow {
                        return LRESULT(1);
                    }
                }
            }

            // 2. Proportional mouse broadcasting
            if MOUSE_BROADCAST_ENABLED.load(Ordering::Relaxed) {
                let fg_hwnd = unsafe { GetForegroundWindow() };
                if act != 0 && fg_hwnd.0 as isize == act {
                    let mut main_client_rect = RECT::default();
                    unsafe { let _ = GetClientRect(fg_hwnd, &mut main_client_rect); };
                    let mut client_pt = pt;
                    unsafe { let _ = ScreenToClient(fg_hwnd, &mut client_pt); };

                    let mw = (main_client_rect.right - main_client_rect.left) as f32;
                    let mh = (main_client_rect.bottom - main_client_rect.top) as f32;

                    if mw > 0.0 && mh > 0.0 {
                        let rx = (client_pt.x as f32 / mw).clamp(0.0, 1.0);
                        let ry = (client_pt.y as f32 / mh).clamp(0.0, 1.0);

                        if let Ok(hwnds) = WOW_HWNDS.read() {
                            for slave in hwnds.iter() {
                                if slave.0 != fg_hwnd {
                                    let mut slave_rect = RECT::default();
                                    unsafe { let _ = GetClientRect(slave.0, &mut slave_rect); };
                                    let sw = (slave_rect.right - slave_rect.left) as f32;
                                    let sh = (slave_rect.bottom - slave_rect.top) as f32;

                                    let target_x = (rx * sw) as i32;
                                    let target_y = (ry * sh) as i32;
                                    let target_lparam = ((target_y as isize) << 16) | ((target_x as isize) & 0xFFFF);

                                    unsafe {
                                        let _ = PostMessageW(
                                            Some(slave.0),
                                            msg,
                                            WPARAM(0),
                                            LPARAM(target_lparam),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    unsafe { CallNextHookEx(None, n_code, wparam, lparam) }
}
