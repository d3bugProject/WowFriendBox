use windows::core::{BOOL, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, EnumWindows, GetClassNameW, GetForegroundWindow,
    GetWindowLongPtrW, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    IsWindowVisible, SetForegroundWindow, SetWindowLongPtrW, SetWindowPos, SetWindowTextW,
    ShowWindow, GWL_STYLE, HWND_TOP, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_SHOW, WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_POPUP,
    WS_SYSMENU, WS_THICKFRAME, WS_VISIBLE,
};

#[link(name = "user32")]
unsafe extern "system" {
    fn AttachThreadInput(id_attach: u32, id_attach_to: u32, f_attach: BOOL) -> BOOL;
}

use crate::config::LayoutConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SafeHwnd(pub HWND);
unsafe impl Send for SafeHwnd {}
unsafe impl Sync for SafeHwnd {}

impl SafeHwnd {
    pub fn as_isize(&self) -> isize {
        self.0.0 as isize
    }
}

#[derive(Debug, Clone)]
pub struct WowClient {
    pub hwnd: SafeHwnd,
    pub pid: u32,
    pub slot: usize, // 1 to 5
    pub title: String,
}

pub struct WindowManager {
    pub clients: Vec<WowClient>,
    pub active_slot: usize,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
            active_slot: 1,
        }
    }

    /// Continuous scan for WoW windows.
    /// Returns true if any window was added or removed.
    pub fn refresh_clients(&mut self, config: &LayoutConfig) -> bool {
        let mut discovered_hwnds: Vec<HWND> = Vec::new();

        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let list = unsafe { &mut *(lparam.0 as *mut Vec<HWND>) };
            if unsafe { IsWindowVisible(hwnd) }.as_bool() {
                let mut is_wow = false;

                // 1. Check window class name (GxWindowClass)
                let mut class_buf = [0u16; 256];
                let len = unsafe { GetClassNameW(hwnd, &mut class_buf) };
                if len > 0 {
                    let class_name = String::from_utf16_lossy(&class_buf[..len as usize]);
                    if class_name.to_lowercase().contains("gxwindow") {
                        is_wow = true;
                    }
                }

                // 2. Fallback: check window title
                if !is_wow {
                    let title_len = unsafe { GetWindowTextLengthW(hwnd) };
                    if title_len > 0 {
                        let mut title_buf = vec![0u16; (title_len + 1) as usize];
                        let read_len = unsafe { GetWindowTextW(hwnd, &mut title_buf) };
                        let title = String::from_utf16_lossy(&title_buf[..read_len as usize]);
                        if title.to_lowercase().contains("world of warcraft") {
                            is_wow = true;
                        }
                    }
                }

                if is_wow {
                    list.push(hwnd);
                }
            }
            BOOL(1)
        }

        unsafe {
            let _ = EnumWindows(
                Some(enum_proc),
                LPARAM(&mut discovered_hwnds as *mut Vec<HWND> as isize),
            );
        }

        // Compare current list with discovered list
        let old_hwnds: Vec<HWND> = self.clients.iter().map(|c| c.hwnd.0).collect();
        let changed = old_hwnds.len() != discovered_hwnds.len()
            || !discovered_hwnds.iter().all(|h| old_hwnds.contains(h));

        if !changed {
            return false;
        }

        // 1. Remove closed windows
        self.clients.retain(|c| discovered_hwnds.contains(&c.hwnd.0));

        // 2. Add newly discovered windows
        for hwnd in discovered_hwnds {
            if !self.clients.iter().any(|c| c.hwnd.0 == hwnd) {
                let mut pid = 0u32;
                unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)); }

                // Check if launcher assigned this PID to a specific slot
                let assigned_slot = if let Ok(lock) = crate::launcher::LAUNCHED_PIDS.read() {
                    lock.get(&pid).copied()
                } else {
                    None
                };

                let next_slot = if let Some(s) = assigned_slot {
                    s
                } else {
                    let mut s = 1;
                    while self.clients.iter().any(|c| c.slot == s) {
                        s += 1;
                    }
                    s
                };

                // Strip borders once on discovery
                if config.remove_borders {
                    self.make_borderless(hwnd);
                }

                // Rename window title on taskbar for clear identification
                let desc = slot_description(next_slot);
                let title = format!("WoW Slot #{} ({})", next_slot, desc);
                let wide_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
                unsafe {
                    let _ = SetWindowTextW(hwnd, PCWSTR(wide_title.as_ptr()));
                }

                self.clients.push(WowClient {
                    hwnd: SafeHwnd(hwnd),
                    pid,
                    slot: next_slot,
                    title,
                });
            }
        }

        // 3. Ensure active_slot points to a valid slot
        if self.clients.is_empty() {
            self.active_slot = 1;
        } else if !self.clients.iter().any(|c| c.slot == self.active_slot) {
            self.active_slot = self.clients[0].slot;
        }

        // Sort by slot index
        self.clients.sort_by_key(|c| c.slot);
        true
    }

    /// Removes window borders and titlebar for clean borderless mode
    pub fn make_borderless(&self, hwnd: HWND) {
        unsafe {
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            let border_styles = (WS_CAPTION.0 | WS_THICKFRAME.0 | WS_MINIMIZEBOX.0 | WS_MAXIMIZEBOX.0 | WS_SYSMENU.0) as isize;
            let new_style = (style & !border_styles) | (WS_POPUP.0 | WS_VISIBLE.0) as isize;
            let _ = SetWindowLongPtrW(hwnd, GWL_STYLE, new_style);
        }
    }

    /// Positions all windows:
    /// - Active Slot -> Center Foreground: 1500x843px
    /// - Background Grid 4 Slots (each 1280x540px):
    ///   * Top-Left:     (x = 0, y = 0)
    ///   * Bottom-Left:  (x = 0, y = 540)
    ///   * Top-Right:    (x = 1280, y = 0)
    ///   * Bottom-Right: (x = 1280, y = 540)
    pub fn apply_layout(&mut self, config: &LayoutConfig) {
        if self.clients.is_empty() {
            return;
        }

        let sw = config.screen_resolution[0];
        let sh = config.screen_resolution[1];

        if config.mode.to_lowercase() == "pip" {
            let cw = config.center_window_size[0];
            let ch = config.center_window_size[1];

            let center_x = (sw - cw) / 2;
            let center_y = (sh - ch) / 2;

            let half_w = sw / 2;
            let half_h = sh / 2;

            let top_left     = (0, 0, half_w, half_h);
            let bottom_left  = (0, half_h, half_w, half_h);
            let top_right    = (half_w, 0, half_w, half_h);
            let bottom_right = (half_w, half_h, half_w, half_h);

            let get_background_rect = |slot: usize, active: usize| -> (i32, i32, i32, i32) {
                let target_spot = if slot == 1 { active } else { slot };
                match target_spot {
                    2 => top_left,
                    3 => bottom_left,
                    4 => top_right,
                    5 => bottom_right,
                    _ => top_left,
                }
            };

            for client in &self.clients {
                if client.slot != self.active_slot {
                    let (x, y, w, h) = get_background_rect(client.slot, self.active_slot);
                    unsafe {
                        let _ = SetWindowPos(
                            client.hwnd.0,
                            None,
                            x,
                            y,
                            w,
                            h,
                            SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
                        );
                    }
                }
            }

            if let Some(active_client) = self.clients.iter().find(|c| c.slot == self.active_slot) {
                unsafe {
                    let _ = SetWindowPos(
                        active_client.hwnd.0,
                        Some(HWND_TOP),
                        center_x,
                        center_y,
                        cw,
                        ch,
                        SWP_FRAMECHANGED | SWP_SHOWWINDOW,
                    );
                }
                force_foreground_window(active_client.hwnd.0);
            }
        } else {
            // Stacked Single-Window Mode (Zero screen clutter, no mouse blocking, instant swap)
            for client in &self.clients {
                if client.slot != self.active_slot {
                    unsafe {
                        let _ = SetWindowPos(
                            client.hwnd.0,
                            None,
                            0,
                            0,
                            sw,
                            sh,
                            SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
                        );
                    }
                }
            }

            if let Some(active_client) = self.clients.iter().find(|c| c.slot == self.active_slot) {
                unsafe {
                    let _ = SetWindowPos(
                        active_client.hwnd.0,
                        Some(HWND_TOP),
                        0,
                        0,
                        sw,
                        sh,
                        SWP_FRAMECHANGED | SWP_SHOWWINDOW,
                    );
                }
                force_foreground_window(active_client.hwnd.0);
            }
        }
    }

    /// Swaps the specified slot to the foreground
    pub fn swap_to_slot(&mut self, target_slot: usize, config: &LayoutConfig) {
        if !self.clients.iter().any(|c| c.slot == target_slot) {
            return;
        }

        self.active_slot = target_slot;

        if config.mode.to_lowercase() == "pip" {
            self.apply_layout(config);
        } else {
            // Stacked mode: zero resizing, instant BringWindowToTop
            if let Some(target) = self.clients.iter().find(|c| c.slot == target_slot) {
                unsafe {
                    let _ = SetWindowPos(
                        target.hwnd.0,
                        Some(HWND_TOP),
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
                    );
                }
                force_foreground_window(target.hwnd.0);
            }
        }
    }
}

/// Forces Windows to transfer keyboard and mouse focus to the target window
pub fn force_foreground_window(target_hwnd: HWND) {
    unsafe {
        let fg_hwnd = GetForegroundWindow();
        if fg_hwnd == target_hwnd {
            return;
        }

        let cur_thread = GetCurrentThreadId();
        let mut fg_pid = 0u32;
        let fg_thread = GetWindowThreadProcessId(fg_hwnd, Some(&mut fg_pid));
        let mut target_pid = 0u32;
        let target_thread = GetWindowThreadProcessId(target_hwnd, Some(&mut target_pid));

        // Attach input threads to bypass Windows foreground restrictions
        let mut attached_fg = false;
        let mut attached_target = false;

        if fg_thread != 0 && fg_thread != cur_thread {
            attached_fg = AttachThreadInput(cur_thread, fg_thread, BOOL(1)).as_bool();
        }
        if target_thread != 0 && target_thread != cur_thread {
            attached_target = AttachThreadInput(cur_thread, target_thread, BOOL(1)).as_bool();
        }

        let _ = ShowWindow(target_hwnd, SW_SHOW);
        let _ = BringWindowToTop(target_hwnd);
        let _ = SetForegroundWindow(target_hwnd);

        if attached_target {
            let _ = AttachThreadInput(cur_thread, target_thread, BOOL(0));
        }
        if attached_fg {
            let _ = AttachThreadInput(cur_thread, fg_thread, BOOL(0));
        }
    }
}

pub fn slot_description(slot: usize) -> &'static str {
    match slot {
        1 => "Center Main Window",
        2 => "Top-Left Window (1280x540)",
        3 => "Bottom-Left Window (1280x540)",
        4 => "Top-Right Window (1280x540)",
        5 => "Bottom-Right Window (1280x540)",
        _ => "Unknown Slot",
    }
}
