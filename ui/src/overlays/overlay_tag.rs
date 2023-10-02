use eframe::egui::{Ui, Color32};
use tickets_rs_core::Tag;

use crate::{
    Overlay, 
    UITheme, 
    UIController, UICache
};

use super::{
    OverlayHelper, 
    OverlayAction,  
    DialogOptions
};


#[derive(Default, PartialEq, Clone)]
pub struct NewTagData {
    pub tag: Tag,
    pub font_color: Color32,
    pub back_color: Color32,
    pub adapters: Vec<(String, String)>,
    pub errors: Vec<(String, String)>,
}

impl Overlay {

    pub(crate) fn update_new_tag(
        ui: &mut Ui,
        ui_theme: &mut UITheme,
        tag_data: &mut NewTagData
    ) -> OverlayAction {
        OverlayHelper::helper_update_header(ui, ui_theme, "New Tag");

        OverlayHelper::helper_update_spacer(ui, ui_theme);
        OverlayHelper::helper_update_spacer(ui, ui_theme);
        OverlayHelper::helper_update_tag(ui, ui_theme, &tag_data.tag.name, &tag_data.font_color, &tag_data.back_color);
        OverlayHelper::helper_update_spacer(ui, ui_theme);
        OverlayHelper::helper_update_spacer(ui, ui_theme);

        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Main Content", true, |ui| {
            OverlayHelper::helper_update_adapter(ui, ui_theme, &mut tag_data.tag.adapter, &tag_data.adapters);
            OverlayHelper::helper_update_small_spacer(ui, ui_theme);
            OverlayHelper::helper_update_text(ui, ui_theme, &mut tag_data.tag.name, "Name:");
        });

        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Appearance", false, |ui| {
            OverlayHelper::helper_update_color(ui, ui_theme, "Front:", &mut tag_data.font_color);
            OverlayHelper::helper_update_small_spacer(ui, ui_theme);
            OverlayHelper::helper_update_color(ui, ui_theme, "Back:", &mut tag_data.back_color);
        });

        OverlayHelper::helper_update_small_spacer(ui, ui_theme);
        OverlayHelper::helper_update_errors(ui, ui_theme, &tag_data.errors);
        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Create Tag".to_string())) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            DialogOptions::Close => OverlayAction::CloseOverlay,
            DialogOptions::Confirm => OverlayAction::NewTag(
                tag_data.tag.clone()
                .with_hex_colors(
                    UIController::color_as_string(tag_data.back_color).as_str(), 
                    UIController::color_as_string(tag_data.font_color).as_str())),
        }
    }
    
}

impl OverlayAction {
    pub(crate) fn action_tag(
        ui_controller: &mut UIController,
        cache: &mut UICache,
        tag: Tag
    ) {
        ui_controller.using_ticket_provider_mut(|controller, provider| {
            match provider.tag_validate(&tag) {
                Ok(_) => {
                    match provider.tag_write(&tag) {
                        Ok(_) => {
                            cache.tags_valid = false;
                            controller.close_overlay()
                        },
                        Err(error) => {

                            let error_message = error.get_text();
                            let mut errors = vec![("other".to_string(), error_message)];

                            Overlay::put_errors(&mut controller.get_current_overlay(), &mut errors);

                        },
                    };
                },
                Err(adapter_error) => {

                    let mut errors = match adapter_error.error_type {
                        tickets_rs_core::AdapterErrorType::Validate(errors_vec, _) => errors_vec,
                        _ => {
                            let error_message = adapter_error.get_text();
                            vec![("other".to_string(), error_message)]
                        }
                    };

                    Overlay::put_errors(&mut controller.get_current_overlay(), &mut errors);
                }
            }
        });
    }

    pub(crate) fn action_tag_delete(
        ui_controller: &mut UIController,
        cache: &mut UICache,
        tag: Tag
    ) {
        let mut action_successful: bool = false;
        ui_controller.using_ticket_provider_mut(|controller, provider| {

            match provider.tag_drop(&tag) {
                Ok(_) => action_successful = true,
                Err(error) => {

                    let error_message = error.get_text();
                    let mut errors = vec![("other".to_string(), error_message)];

                    Overlay::put_errors(controller.get_current_overlay(), &mut errors);

                },
            };

        });

        if action_successful {
            ui_controller.close_overlay();
            cache.tags_valid = false;
        }
    }
}