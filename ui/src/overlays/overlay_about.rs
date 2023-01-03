use std::collections::{
    HashMap, 
    hash_map::RandomState
};

use egui::{
    Ui, 
    ColorImage, 
    TextureHandle, 
    RichText
};

use crate::{
    UITheme, 
    Overlay
};

use super::{
    OverlayHelper, 
    OverlayAction, 
    DialogOptions
};


impl Overlay {

    pub(crate) fn update_about(
        ui: &mut Ui, 
        ui_theme: &UITheme,
        icon_textures: &mut HashMap<String, Option<TextureHandle>, RandomState>,
        icons: &mut HashMap<String, Option<ColorImage>, RandomState>
    ) -> OverlayAction {
        OverlayHelper::helper_update_header(ui, ui_theme, "About");
        OverlayHelper::helper_update_spacer(ui, ui_theme);
        OverlayHelper::helper_update_icon(ui, icon_textures, icons, &"icon_app".to_string(), 120.0);
        OverlayHelper::helper_update_spacer(ui, ui_theme);
        ui.label(RichText::new("tickets.rs - A Ticket Management App").strong());
        OverlayHelper::helper_update_small_spacer(ui, ui_theme);
        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Details", true, |ui| {
            OverlayHelper::helper_update_card(ui, ui_theme, "Devs".to_string(), |ui| {
                ui.label("Robert Lang");
            });
    
            OverlayHelper::helper_update_card(ui, ui_theme, "Version".to_string(), |ui| {
                ui.label("tickets.rs v2023.01");
            });
    
            OverlayHelper::helper_update_card(ui, ui_theme, "History".to_string(), |ui| {
                ui.label("Development of prototype has been started in September 2021.");
                ui.label("Prototype finished in December 2021, written in Python 3.6");
                ui.label("Testing of the prototype has been continued until October 2022");
                ui.label("The development of the Release Version in Rust has been started in October 2022");
            });
            
        });
    
        OverlayHelper::helper_update_spacer(ui, ui_theme);
        ui.label(RichText::new("The software works as-is, and doesn't need a license.").strong());
        OverlayHelper::helper_update_spacer(ui, ui_theme);
    
        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, None) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            DialogOptions::Close => OverlayAction::CloseOverlay,
            DialogOptions::Confirm => OverlayAction::CloseOverlay,
        }
    }

}