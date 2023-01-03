use egui::Ui;
use tickets_rs_core::Config;
use std::fmt;

use crate::{Overlay, UITheme, ui_controller, UIController, UICache};

use super::{OverlayHelper, OverlayAction, DialogOptions};


#[derive(Default, PartialEq, Clone)]
pub struct WizardData {
    pub page: u8,
    pub extension_config: Option<Config>,
    pub username: String
}

impl Overlay {

    pub(crate) fn update_wizard(
        ui: &mut Ui,
        ui_theme: &mut UITheme,
        ui_controller: &mut UIController,
        wizard_data: &mut WizardData,
    ) -> OverlayAction {
        OverlayHelper::helper_update_header(ui, ui_theme, format!("Setup Wizard ({}/3)", wizard_data.page + 1u8).as_str() );

        match wizard_data.page {
            0 => Overlay::update_wizard_0(ui, ui_theme, ui_controller, wizard_data),
            1 => Overlay::update_wizard_1(ui, ui_theme, ui_controller, wizard_data),
            _ => Overlay::update_wizard_2(ui, ui_theme, ui_controller, wizard_data),
        }

        if wizard_data.page < 2u8 {
            match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Next".to_string())) {
                DialogOptions::Nothing => OverlayAction::Nothing,
                DialogOptions::Close => OverlayAction::CloseOverlay,
                DialogOptions::Confirm => {
                    wizard_data.page += 1;
                    OverlayAction::Nothing
                },
            }
        } else {
            match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Finish".to_string())) {
                DialogOptions::Nothing => OverlayAction::Nothing,
                DialogOptions::Close => OverlayAction::CloseOverlay,
                DialogOptions::Confirm => OverlayAction::WizardDone(wizard_data.clone()),
            }
        }
        
    }

    fn update_wizard_0(
        ui: &mut Ui,
        ui_theme: &UITheme,
        ui_controller: &mut UIController,
        wizard_data: &mut WizardData,
    ) {
        let label_text = 
        ["Welcome to tickets.rs!", "",
        "This Wizard is here to prepare all necessary data for you to easily being able to use this tool.",
        "If you accidentally close this overlay, you can reopen it under Help -> Open Wizard.", "",
        "Let's start; What is the name, you want to use in tasks and tickets assigned to yourself?"].join("\n");
        ui.label(label_text);

        OverlayHelper::helper_update_spacer(ui, ui_theme);

        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "General Info", true, |ui| {
            OverlayHelper::helper_update_text(ui, ui_theme, &mut wizard_data.username, "Username:");
        });
    }

    fn update_wizard_1(
        ui: &mut Ui,
        mut ui_theme: &mut UITheme,
        ui_controller: &mut UIController,
        wizard_data: &mut WizardData,
    ) {
        let label_text = 
        ["Thanks for the Name! It will be saved locally, so you don't need to worry it being uploaded somewhere",
        "Let's look into the appearance of the Tool next.",
        "Do you prefer a light theme, or a dark theme? What is your preferred Font size?"].join("\n");
        ui.label(label_text);

        let ui_theme_copy = ui_theme.clone();

        OverlayHelper::helper_update_spacer(ui, &ui_theme_copy);

        OverlayHelper::helper_update_section_collapsing(ui, &ui_theme_copy, "Appearance", true, |ui| {
            let old_font_size = ui_theme.font_size;
            OverlayHelper::helper_update_theme(ui, &ui_theme_copy, &mut ui_theme, ui_controller);
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
    }

    fn update_wizard_2(
        ui: &mut Ui,
        ui_theme: &UITheme,
        ui_controller: &mut UIController,
        wizard_data: &mut WizardData,
    ) {
        let label_text = 
        ["Okay, that's the appearance!",
        "Now let's finally set up the extensions, you want to use.",
        "Extensions essentially define the functionality you have within this tool, and to other interfaces.", "",
        "Go ahead and choose the ones you need."].join("\n");
        ui.label(label_text);

        OverlayHelper::helper_update_spacer(ui, ui_theme);

        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Extensions", true, |ui| {
            OverlayHelper::helper_update_extensions(ui, ui_theme, ui_controller.ticket_provider.clone(), &mut wizard_data.extension_config);
        });
    }
}

impl OverlayAction {
    pub(crate) fn action_wizard(
        ui_controller: &mut UIController,
        cache: &mut UICache,
        wizard_data: WizardData
    ) {
        ui_controller.close_overlay();
        ui_controller.invalidate_cache(Some(cache));
        ui_controller.update_bucket_panel_data();
        match ui_controller.configuration.lock() {
            Ok(mut config_lock) => {
                config_lock.put("wizard", false, "");
                config_lock.put("username", wizard_data.username, "");
                cache.username_valid = false;

            },
            Err(err) => println!("Wasn't able to access app config at the end of the wizard due to {err}")
        };
    }
}