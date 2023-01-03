mod side_panel;
mod menu_bar;
mod ticket;

use std::{
    sync::Arc, 
    collections::HashMap, 
    path::Path, 
    ops::Deref
};

use eframe::{
    HardwareAcceleration, 
    epaint::Shadow
};

use egui::{
    Vec2, 
    Button, 
    style::Margin,
    Ui, 
    TopBottomPanel, 
    Frame, 
    Rounding, 
    Stroke, 
    SidePanel, 
    ScrollArea, 
    Color32,  
    Align, 
    CentralPanel,  
    TextureHandle, 
    ColorImage, 
    FontId, 
    FontFamily, 
    Area, 
    Align2, 
    Layout, 
    TextStyle
};

use crate::{
    UIController, 
    UITheme, 
    Overlay, overlays::OverlayAction, UICache
};

pub use side_panel::SidePanelAction;


pub struct UserInterface {
    ui_controller: UIController,
    cache: UICache,
    ui_theme: UITheme,
    icons: HashMap<String, Option<ColorImage>>,
    icon_textures: HashMap<String, Option<TextureHandle>>,
}

impl UserInterface {

    pub fn launch(ui_controller: UIController, ui_theme: UITheme) {

        // read the application icon, if possible
        let icon_image = ui_controller.read_image_data_from_path(Path::new("assets/icon_app.png"));
        let icon = match icon_image {
            Some(image) => {
                Some(UIController::image_data_as_icon(image))
            },
            None => None
        };
        
        let options = eframe::NativeOptions {
            always_on_top: false,
            maximized: false,
            decorated: true,
            fullscreen: false,
            drag_and_drop_support: false,
            icon_data: icon,
            //icon_data: None,
            initial_window_pos: None,
            initial_window_size: Some(Vec2{x: 800.0, y: 600.0}),
            min_window_size: Some(Vec2{x: 400.0, y: 200.0}),
            max_window_size: None,
            resizable: true,
            transparent: false,
            vsync: false,
            multisampling: 0,
            depth_buffer: 0,
            stencil_buffer: 0,
            hardware_acceleration: HardwareAcceleration::Preferred,
            renderer: eframe::Renderer::Glow,
            follow_system_theme: false,
            default_theme: ui_theme.base_theme,
            run_and_return: true,
        };

        eframe::run_native(
            "tickets.rs - A ticket Management App",
            options,
            Box::new(|_cc| Box::new(UserInterface::new(ui_controller, ui_theme))),
        );
    }

    pub fn load_texture(icon_textures: & mut HashMap<String, Option<TextureHandle>>, icons: &mut HashMap<String, Option<ColorImage>>, ui: &Ui, adapter_name: &String) -> Option<egui::TextureHandle> {

        match icon_textures.get_mut(adapter_name) {
            Some(texture) => {
                let color_image = icons.get(adapter_name).unwrap();
                match color_image {
                    Some(found_image) => {
                        Some(texture.get_or_insert_with(|| {
                            ui.ctx().load_texture(
                                adapter_name,
                                found_image.clone(),
                                egui::TextureFilter::Linear
                            )
                        })).cloned()
                    },
                    None => None
                }
            },
            None => {
                match icons.get(adapter_name) {
                    Some(found_icon_data_option) => {
                        match found_icon_data_option {
                            Some(found_icon_data) => {
                                icon_textures.insert(adapter_name.clone(), None);
                                //let mut local_tex: Option<TextureHandle> = None;
                                let texture = icon_textures.get_mut(adapter_name).unwrap();
                                //let texture = &mut local_tex;
                                Some(texture.get_or_insert_with(|| {
                                    ui.ctx().load_texture(
                                        adapter_name,
                                        found_icon_data.clone(),
                                        egui::TextureFilter::Linear
                                    )
                                })).cloned()
                            },
                            None => None
                        }
                    },
                    None => None,
                }
            },
        }
    }

    fn new(ui_controller: UIController, ui_theme: UITheme) -> Self {

        let mut icons: HashMap<String, Option<ColorImage>> = HashMap::new();

        ui_controller.read_adapter_icons(&mut icons);
        ui_controller.read_custom_icon(&mut icons, Path::new("assets/icon_app.png"), "icon_app".into());

        UserInterface { 
            icons,
            icon_textures: HashMap::new(),
            cache: UICache::default(),
            ui_controller,
            ui_theme,
        }
    }
}

impl eframe::App for UserInterface {

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        if !self.ui_controller.running {
            self.ui_controller.on_close_ui(&self.ui_theme, frame);
        };

        self.cache.refresh_cache(&mut self.ui_controller);

        let no_color =  Color32::from_rgba_unmultiplied(0, 0, 0, 0);
        let no_shadow = Shadow { extrusion: 0.0, color: no_color };
        let divider_color = Color32::from_rgba_unmultiplied(
            self.ui_theme.background_primary.r(),
            self.ui_theme.background_primary.g(),
            self.ui_theme.background_primary.b(), 
            self.ui_theme.background_primary.a() / 15 * 14);

        if self.ui_controller.font_changed {
            let mut font_update_style = ctx.style().deref().clone();

            font_update_style.text_styles.clear();
            font_update_style.text_styles.insert(TextStyle::Body, FontId { size: self.ui_theme.font_size as f32, family: FontFamily::Proportional });
            font_update_style.text_styles.insert(TextStyle::Monospace, FontId { size: self.ui_theme.font_size as f32, family: FontFamily::Monospace });
            font_update_style.text_styles.insert(TextStyle::Heading, FontId { size: self.ui_theme.font_size as f32 * 1.5, family: FontFamily::Proportional });
            font_update_style.text_styles.insert(TextStyle::Small, FontId { size: self.ui_theme.font_size as f32 * 0.5, family: FontFamily::Proportional });
            font_update_style.text_styles.insert(TextStyle::Button, FontId { size: self.ui_theme.font_size as f32, family: FontFamily::Proportional });
            ctx.set_style(Arc::new(font_update_style));
            self.ui_controller.font_changed = false;
        }

        Area::new("background_area")
        .order(egui::Order::Background)
        .interactable(false)
        .anchor(Align2::LEFT_TOP, Vec2{x: 0.0, y: 0.0})
        .show(ctx, |ui| {
            CentralPanel::default()
                .frame(Frame {
                    inner_margin: Margin::same(0.0),
                    outer_margin: Margin::same(0.0),
                    rounding: Rounding::same(0.0),
                    shadow: no_shadow,
                    fill: self.ui_theme.background_secondary,
                    stroke: Stroke::none(),
                })
                .show_inside(ui, |_| {})
        });

        Area::new("main_area")
        .order(egui::Order::Background)
        .interactable(!self.ui_controller.has_overlay())
        .anchor(Align2::LEFT_TOP, Vec2{x: 0.0, y: 0.0})
        .show(ctx, |ui| {

            TopBottomPanel::top("menu_panel")
                .frame(Frame {
                    inner_margin: Margin{ left: 2.0, right: 2.0, top: 2.0, bottom: 0.0 },
                    outer_margin: Margin::same(0.0),
                    rounding: Rounding::same(0.0),
                    shadow: no_shadow,
                    fill: self.ui_theme.background_primary,
                    stroke: Stroke::none(),
                })
                .show_inside(ui, |ui| {
                    self.update_menu_bar(ctx, ui);
                });

            TopBottomPanel::top("menu_panel_divider")
                .min_height(6.0)
                .max_height(6.0)
                .frame(Frame {
                    inner_margin: Margin::same(0.0),
                    outer_margin: Margin::same(0.0),
                    rounding: Rounding::same(0.0),
                    shadow: no_shadow,
                    fill: self.ui_theme.background_secondary,
                    stroke: Stroke::none(),
                })
                .show_inside(ui, |ui| {});

            if self.ui_controller.show_sidebar {
                SidePanel::left("buckets_panel")
                .default_width(250.0)
                .min_width(100.0)
                .max_width((350.0_f32).min(ui.available_width() - 200.0))
                .frame(Frame {
                    inner_margin: Margin{ left: 12.0, right: 4.0, top: 2.0, bottom: 2.0 },
                    outer_margin: Margin::same(0.0),
                    rounding: Rounding::same(0.0),
                    shadow: no_shadow,
                    fill: self.ui_theme.background_secondary,
                    stroke: Stroke::none(),
                })
                .show_inside(ui, |ui| {
                    self.update_side_panel(ctx, ui)
                });
            } else {
                SidePanel::left("buckets_panel")
                .resizable(false)
                .min_width(5.0)
                .max_width(5.0)
                .frame(Frame {
                    inner_margin: Margin::same(0.0),
                    outer_margin: Margin::same(0.0),
                    rounding: Rounding::same(0.0),
                    shadow: no_shadow,
                    fill: self.ui_theme.background_secondary,
                    stroke: Stroke::none(),
                })
                .show_inside(ui, |ui| {});
            }

            CentralPanel::default()
                .frame(Frame {
                    inner_margin: Margin { left: 8.0, right: 2.0, top: 2.0, bottom: 2.0 },
                    outer_margin: Margin::same(0.0),
                    rounding: Rounding::same(0.0),
                    shadow: no_shadow,
                    fill: self.ui_theme.background_primary,
                    stroke: Stroke::none(),
                })
                .show_inside(ui, |ui| {

                    ScrollArea::vertical()
                        .show(ui, |ui| {
                            ui.add_space(self.ui_theme.font_size as f32 / 2.0);
                            self.ui_controller.update_each_ticket(
                                ui, 
                                &mut self.icon_textures, 
                                &mut self.icons, 
                                &self.ui_theme, 
                                &mut self.cache
                            );
                            //ui.separator();

                        });

                });
        });

        let mut overlay_width = 0.0;

        if self.ui_controller.has_overlay() {
            Area::new("layer_divider")
            .order(egui::Order::Middle)
            .interactable(true)
            .anchor(Align2::LEFT_TOP, Vec2{x: 0.0, y: 0.0})
            .show(ctx, |ui| {
                overlay_width = ui.available_width();
                let space = Button::new("").fill(divider_color).stroke(Stroke{ width: 0.0, color: divider_color });
                if ui.add_sized(ui.available_size(), space).clicked() {
                    self.ui_controller.close_overlay();
                }
            });

            Area::new("overlay_area")
            .order(egui::Order::Foreground)
            .interactable(true)
            .anchor(Align2::CENTER_CENTER, Vec2{x: 0.0, y: 0.0})
            .show(ctx, |ui| {
                ui.set_max_size(Vec2{x: (overlay_width - 64.0).min((self.ui_theme.font_size * 60) as f32), y: ui.available_height() - 64.0});
                ui.set_min_size(Vec2{x: 400.0 - 32.0, y: 200.0 - 32.0});

                let mut overlay_group = Frame::group(ui.style());
                overlay_group = overlay_group.fill(self.ui_theme.background_secondary);
                overlay_group = overlay_group.shadow(ctx.style().visuals.popup_shadow);
                overlay_group = overlay_group.inner_margin(Margin::same(self.ui_theme.font_size as f32));
                overlay_group.show(ui, |ui| {

                    ScrollArea::new([false, true])
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.with_layout(Layout::top_down(Align::Center).with_cross_justify(false), |ui| {
                            ui.style_mut().spacing.item_spacing.y = self.ui_theme.font_size as f32 / 2.0;
                            let mut overlay = self.ui_controller.get_current_overlay().clone();
                            let action = Overlay::update(
                                &mut overlay, 
                                ui, 
                                &mut self.ui_theme, 
                                &mut self.ui_controller,
                                &mut self.cache,
                                &mut self.icon_textures,
                                &mut self.icons
                            );

                            self.ui_controller.open_overlay(overlay);

                            if action != OverlayAction::Nothing {
                                action.execute(&mut self.ui_controller, &mut self.cache)
                            };
                        });
                    });
                });
            });
        }
    }
}
