use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, FontId, Pos2, Rect, Vec2};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN};

use crate::input::{AppEvent, BROADCAST_ENABLED, MOUSE_BROADCAST_ENABLED};
use crate::logger::LOG_ENTRIES;

#[derive(Debug, Clone, Default)]
pub struct GuiClientSlot {
    pub slot: usize,
    pub title: String,
    pub pid: u32,
    pub is_active: bool,
    pub fps: u32,
}

pub static GUI_SLOTS: std::sync::RwLock<Vec<GuiClientSlot>> = std::sync::RwLock::new(Vec::new());

pub struct IsboxerGui {
    event_tx: Sender<AppEvent>,
    textures: HashMap<String, egui::TextureHandle>,
}

impl IsboxerGui {
    pub fn new(cc: &eframe::CreationContext<'_>, event_tx: Sender<AppEvent>) -> Self {
        setup_fonts(&cc.egui_ctx);

        Self {
            event_tx,
            textures: HashMap::new(),
        }
    }

    fn get_texture(&mut self, ctx: &egui::Context, path: &str) -> Option<&egui::TextureHandle> {
        if !self.textures.contains_key(path) {
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let size = [img.width() as usize, img.height() as usize];
                    let rgba = img.to_rgba8();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_flat_samples().as_slice());
                    let handle = ctx.load_texture(path, color_image, egui::TextureOptions::LINEAR);
                    self.textures.insert(path.to_string(), handle);
                }
            }
        }
        self.textures.get(path)
    }

    fn draw_image_btn(&mut self, ui: &mut egui::Ui, path: &str, fallback_text: &str, size: Vec2) -> bool {
        let ctx = ui.ctx().clone();
        if let Some(tex) = self.get_texture(&ctx, path) {
            let img = egui::Image::new(tex)
                .fit_to_exact_size(size)
                .sense(egui::Sense::click());
            ui.add(img).clicked()
        } else {
            ui.button(fallback_text).clicked()
        }
    }
}

fn setup_fonts(ctx: &egui::Context) {
    let font_path = "assets/fonts/custom_font.ttf";
    if let Ok(font_bytes) = std::fs::read(font_path) {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "custom_font".to_owned(),
            Arc::new(FontData::from_owned(font_bytes)),
        );
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "custom_font".to_owned());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "custom_font".to_owned());
        ctx.set_fonts(fonts);
    }
}

impl eframe::App for IsboxerGui {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Fully transparent background so custom PNG frame transparency shows through
        [0.0, 0.0, 0.0, 0.0]
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.request_repaint_after(std::time::Duration::from_millis(150));

        let screen_rect = ui.max_rect();
        let painter = ui.painter();

        // 1. Draw custom background frame (assets/frame.png or assets/background.png)
        let frame_tex_id = if let Some(tex) = self.get_texture(&ctx, "assets/frame.png") {
            Some(tex.id())
        } else if let Some(tex) = self.get_texture(&ctx, "assets/background.png") {
            Some(tex.id())
        } else {
            None
        };

        if let Some(tex_id) = frame_tex_id {
            painter.image(
                tex_id,
                screen_rect,
                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            painter.rect_filled(screen_rect, 6.0, Color32::from_rgba_unmultiplied(20, 24, 32, 230));
        }

        // Window drag handling when clicking on background
        let drag_resp = ui.interact(screen_rect, ui.auto_id_with("hud_window_drag"), egui::Sense::drag());
        if drag_resp.drag_started_by(egui::PointerButton::Primary) {
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        // 2. Main content area: 900x80 px (margins: x=50, y=10)
        let inner_rect = Rect::from_min_size(
            Pos2::new(screen_rect.min.x + 50.0, screen_rect.min.y + 10.0),
            Vec2::new(900.0, 80.0),
        );

        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(inner_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );

        child_ui.horizontal(|ui| {
            // SECTION 1: Logs (450px)
            ui.allocate_ui(Vec2::new(450.0, 80.0), |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("JOURNAL DES LOGS")
                            .font(FontId::monospace(9.0))
                            .color(Color32::from_rgb(0, 220, 255)),
                    );

                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .stick_to_bottom(true)
                        .max_height(60.0)
                        .show(ui, |ui| {
                            if let Ok(logs) = LOG_ENTRIES.read() {
                                for line in logs.iter().rev().take(15).rev() {
                                    ui.label(
                                        egui::RichText::new(line)
                                            .font(FontId::monospace(8.5))
                                            .color(Color32::from_rgb(205, 220, 230)),
                                    );
                                }
                            }
                        });
                });
            });

            ui.separator();

            // SECTION 2: Control Buttons (320px)
            ui.allocate_ui(Vec2::new(320.0, 80.0), |ui| {
                ui.vertical(|ui| {
                    // Top row: Header + Close & Minimize
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("CONTROLES")
                                .font(FontId::monospace(9.0))
                                .color(Color32::from_rgb(0, 255, 180)),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if self.draw_image_btn(ui, "assets/buttons/btn_close.png", "[X]", Vec2::new(16.0, 16.0)) {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            if self.draw_image_btn(ui, "assets/buttons/btn_minimize.png", "[-]", Vec2::new(16.0, 16.0)) {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            }
                        });
                    });

                    ui.add_space(2.0);

                    // Middle row: Toggles & Realign
                    ui.horizontal(|ui| {
                        let key_on = BROADCAST_ENABLED.load(Ordering::Relaxed);
                        let key_path = if key_on {
                            "assets/buttons/btn_keyboard_on.png"
                        } else {
                            "assets/buttons/btn_keyboard_off.png"
                        };
                        let key_lbl = if key_on { "KEY: ON" } else { "KEY: OFF" };
                        if self.draw_image_btn(ui, key_path, key_lbl, Vec2::new(64.0, 22.0)) {
                            let new_val = !key_on;
                            BROADCAST_ENABLED.store(new_val, Ordering::Relaxed);
                            crate::logger::log_event(format!(
                                "[CLAVIER] Broadcast {}",
                                if new_val { "ACTIVE (ON)" } else { "DESACTIVE (OFF)" }
                            ));
                        }

                        let mouse_on = MOUSE_BROADCAST_ENABLED.load(Ordering::Relaxed);
                        let mouse_path = if mouse_on {
                            "assets/buttons/btn_mouse_on.png"
                        } else {
                            "assets/buttons/btn_mouse_off.png"
                        };
                        let mouse_lbl = if mouse_on { "MOU: ON" } else { "MOU: OFF" };
                        if self.draw_image_btn(ui, mouse_path, mouse_lbl, Vec2::new(64.0, 22.0)) {
                            let new_val = !mouse_on;
                            MOUSE_BROADCAST_ENABLED.store(new_val, Ordering::Relaxed);
                            crate::logger::log_event(format!(
                                "[SOURIS] Broadcast {}",
                                if new_val { "ACTIVEE (ON)" } else { "DESACTIVEE (OFF)" }
                            ));
                        }

                        if self.draw_image_btn(ui, "assets/buttons/btn_realign.png", "GRID", Vec2::new(64.0, 22.0)) {
                            let _ = self.event_tx.send(AppEvent::ReapplyLayout);
                        }
                    });

                    ui.add_space(2.0);

                    // Bottom row: F1..F5 Slot Swap Buttons
                    ui.horizontal(|ui| {
                        for slot in 1..=5 {
                            let btn_path = format!("assets/buttons/btn_f{}.png", slot);
                            let label = format!("F{}", slot);
                            if self.draw_image_btn(ui, &btn_path, &label, Vec2::new(28.0, 18.0)) {
                                let _ = self.event_tx.send(AppEvent::Swap(slot));
                            }
                        }
                    });
                });
            });

            ui.separator();

            // SECTION 3: FPS Extension (100px)
            ui.allocate_ui(Vec2::new(100.0, 80.0), |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("FPS WATCH")
                            .font(FontId::monospace(9.0))
                            .color(Color32::from_rgb(255, 205, 0)),
                    );

                    if let Ok(slots) = GUI_SLOTS.read() {
                        if slots.is_empty() {
                            ui.label(
                                egui::RichText::new("En attente WoW...")
                                    .font(FontId::monospace(8.0))
                                    .color(Color32::GRAY),
                            );
                        } else {
                            for s in slots.iter().take(5) {
                                let color = if s.is_active {
                                    Color32::from_rgb(0, 255, 120)
                                } else {
                                    Color32::from_rgb(170, 185, 195)
                                };
                                let role = if s.is_active { "★" } else { " " };
                                let text = format!("W{}: {} fps {}", s.slot, s.fps, role);
                                ui.label(
                                    egui::RichText::new(text)
                                        .font(FontId::monospace(8.0))
                                        .color(color),
                                ).on_hover_text(format!("Slot {}: {} (PID: {})", s.slot, s.title, s.pid));
                            }
                        }
                    }
                });
            });
        });
    }
}

pub fn run_gui(event_tx: Sender<AppEvent>) -> eframe::Result<()> {
    let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) } as f32;
    let pos_x = ((screen_w - 1000.0) / 2.0).max(0.0);
    let pos_y = 5.0;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 100.0])
            .with_position([pos_x, pos_y])
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "ISBoxer-RS HUD",
        native_options,
        Box::new(|cc| Ok(Box::new(IsboxerGui::new(cc, event_tx)))),
    )
}
