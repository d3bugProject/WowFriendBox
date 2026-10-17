# ISBoxer-RS (Multiboxing Controller for Warmane WotLK 3.3.5a)

High-performance, lightweight multiboxing software written in Rust for **World of Warcraft 3.3.5a**.

---

## 🎯 Screen Layout Setup

Designed specifically for your **2560x1080** screen:

1. **Center Foreground Window (1500 x 843 px)**:
   - Centered on screen at `x = 530, y = 118`.
   - Reserved for your active character (Slot 1 by default).
   - Stays in the foreground (`HWND_TOP`).

2. **Background 2x2 Grid (4 Windows of 1280 x 540 px each)**:
   - Fills the entire 2560x1080 background seamlessly without gaps:
     - **Top-Left**: `x = 0, y = 0, w = 1280, h = 540` (Slot 2)
     - **Bottom-Left**: `x = 0, y = 540, w = 1280, h = 540` (Slot 3)
     - **Top-Right**: `x = 1280, y = 0, w = 1280, h = 540` (Slot 4)
     - **Bottom-Right**: `x = 1280, y = 540, w = 1280, h = 540` (Slot 5)

3. **Incremental Slot Assignment**:
   - 1st window opened $\rightarrow$ Center (Slot 1).
   - 2nd window opened $\rightarrow$ Top-Left (Slot 2).
   - 3rd window opened $\rightarrow$ Bottom-Left (Slot 3).
   - 4th window opened $\rightarrow$ Top-Right (Slot 4).
   - 5th window opened $\rightarrow$ Bottom-Right (Slot 5).

4. **Instant Window Swap (`F1` to `F5`)**:
   - Press **F2**: Window #2 moves to the **Center**, while Window #1 takes Window #2's spot in the background (Top-Left).
   - Press **F3**: Window #3 moves to the **Center**, while Window #1 takes Window #3's spot (Bottom-Left).
   - Press **F1**: Window #1 returns to the **Center**, and all background windows return to their default slots!

5. **Anti-Focus Stealing Protection**:
   - Accidental mouse clicks outside the 1500x843 center frame on background WoW windows are **automatically blocked**.
   - Your background windows will **NEVER steal focus** by clicking on them.
   - Window switching is exclusively triggered by **`F1` through `F5`**.

---

## ⌨️ Controls & Hotkeys

| Hotkey | Action | Target Position |
| :--- | :--- | :--- |
| **F1** | Swap **Main Center Window** (Slot 1) to Center | Always Center |
| **F2** | Swap **Top-Left Window** (Slot 2) to Center | Top-Left (1280x540) |
| **F3** | Swap **Bottom-Left Window** (Slot 3) to Center | Bottom-Left (1280x540) |
| **F4** | Swap **Top-Right Window** (Slot 4) to Center | Top-Right (1280x540) |
| **F5** | Swap **Bottom-Right Window** (Slot 5) to Center | Bottom-Right (1280x540) |
| **Scroll Lock** | Toggle Key Broadcasting (ON / OFF) | All accounts |
| **Pause** | Toggle Mouse Broadcasting (ON / OFF) | Proportional clicks |
| **Home** | Re-align & Re-apply the entire layout | Full layout reset |

---

## ⚙️ Configuration (`config.json`)

You can edit `config.json` with Notepad at any time to customize your keys:
- `hotkeys`: Change `toggle_broadcast`, `slot_keys`, etc.
- `broadcast.whitelist`: Keys that duplicate to background windows (`1` to `=`, NumPad keys).
- `broadcast.blacklist`: Keys that are NEVER broadcasted (`W`, `A`, `S`, `D`, `Z`, `Q`, `Space`, `Return`, `Tab`, `Escape`) to prevent desynchronization.
- `layout`: Customize window size and screen resolution.
