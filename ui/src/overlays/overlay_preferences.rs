use egui::Ui;

use crate::{Overlay, UITheme, ui_controller, UIController, UICache};

use super::{OverlayAction, helper::OverlayHelper, DialogOptions};

use tickets_rs_core::Config;


#[derive(Default, PartialEq, Clone)]
pub struct PreferenceData {
    pub username: String,
    pub extension_config: Option<Config>,
}

impl Overlay {
    pub(crate) fn update_preferences(ui: &mut Ui, mut ui_theme: &mut UITheme, ui_controller: &mut UIController, preference_data: &mut PreferenceData) -> OverlayAction {
        OverlayHelper::helper_update_header(ui, ui_theme, "Preferences");

        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "General Info", false, |ui| {
            OverlayHelper::helper_update_text(ui, ui_theme, &mut preference_data.username, "Username:");
        });

        let ui_theme_copy = ui_theme.clone();

        OverlayHelper::helper_update_section_collapsing(ui, &ui_theme_copy, "Appearance", false, |ui| {
            let old_font_size = ui_theme.font_size;
            OverlayHelper::helper_update_theme(ui, &ui_theme_copy, ui_theme, ui_controller);
            ui_theme.font_size = old_font_size;

            if OverlayHelper::helper_update_number(ui, &ui_theme_copy, &mut ui_theme.font_size, "Font size:") {
                ui_controller.font_changed = true;
            };
        });

        OverlayHelper::helper_update_section_collapsing(ui, &ui_theme_copy, "Additional Appearance Details", false, |ui| {
            OverlayHelper::helper_update_color(ui, &ui_theme_copy, "Primary Background", &mut ui_theme.background_primary);
            OverlayHelper::helper_update_color(ui, &ui_theme_copy, "Secondary Background", &mut ui_theme.background_secondary);
            OverlayHelper::helper_update_color(ui, &ui_theme_copy, "Tertiary Background", &mut ui_theme.background_tertiary);
            OverlayHelper::helper_update_color(ui, &ui_theme_copy, "Error Background", &mut ui_theme.background_error);
            OverlayHelper::helper_update_spacer(ui, &ui_theme_copy);
            OverlayHelper::helper_update_color(ui, &ui_theme_copy, "Primary Text", &mut ui_theme.foreground_primary);
            OverlayHelper::helper_update_color(ui, &ui_theme_copy, "Secondary Text", &mut ui_theme.foreground_secondary);
            OverlayHelper::helper_update_color(ui, &ui_theme_copy, "Tertiary Text", &mut ui_theme.foreground_tertiary);
            OverlayHelper::helper_update_color(ui, &ui_theme_copy, "Marker Text", &mut ui_theme.foreground_marker);
            OverlayHelper::helper_update_color(ui, &ui_theme_copy, "Second Marker Text", &mut ui_theme.foreground_marker2);
        });

        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Extensions", false, |ui| {
            OverlayHelper::helper_update_extensions(ui, ui_theme, ui_controller.ticket_provider.clone(), &mut preference_data.extension_config);
        });

        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, None) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            _ => OverlayAction::PreferencesApply(preference_data.clone()),
        }
    }
}

impl OverlayAction {
    pub(crate) fn action_preferences(ui_controller: &mut UIController, cache: &mut UICache, preference_data: PreferenceData) {
        ui_controller.close_overlay();
        ui_controller.invalidate_cache(Some(cache));
        ui_controller.update_bucket_panel_data();
        match ui_controller.configuration.lock() {
            Ok(mut config_lock) => {
                config_lock.put("username", preference_data.username, "");
                cache.username_valid = false;
            },
            Err(err) => println!("Wasn't able to access app config at the end of the preferences due to {err}")
        };
    }
}