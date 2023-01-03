use egui::Ui;
use tickets_rs_core::Filter;

use crate::{Overlay, UITheme, UICache, UIController};

use super::{OverlayAction, helper::OverlayHelper, DialogOptions};

#[derive(Default, PartialEq, Clone)]
pub struct NewFilterData {
    pub filter: Filter,
    pub adapters: Vec<(String, String)>,
    pub errors: Vec<(String, String)>,
}

#[derive(Default, PartialEq, Clone)]
pub struct EditFilterData {
    pub filter: Filter,
    pub errors: Vec<(String, String)>,
}

#[derive(Default, PartialEq, Clone)]
pub struct DeleteFilterData {
    pub errors: Vec<(String, String)>,
    pub filter: Filter
}

impl Overlay {
    pub(crate) fn update_new_filter(
        ui: &mut Ui,
        ui_theme: &UITheme,
        filter_data: &mut NewFilterData
    ) -> OverlayAction {
        OverlayHelper::helper_update_header(ui, ui_theme, "New Filter");

        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Location", true, |ui| {
            OverlayHelper::helper_update_adapter(ui, ui_theme, &mut filter_data.filter.identifier.adapter, &filter_data.adapters);
        });

        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Main Content", true, |ui| {
            OverlayHelper::helper_update_text(ui, ui_theme, &mut filter_data.filter.identifier.name, "Name:");
            OverlayHelper::helper_update_small_spacer(ui, ui_theme);
            OverlayHelper::helper_update_desc(ui, ui_theme, &mut filter_data.filter.operation, false);
        });

        OverlayHelper::helper_update_small_spacer(ui, ui_theme);
        OverlayHelper::helper_update_errors(ui, ui_theme, &filter_data.errors);

        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Create".to_string())) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            DialogOptions::Close => OverlayAction::CloseOverlay,
            DialogOptions::Confirm => OverlayAction::NewFilter(filter_data.filter.clone()),
        }
    }

    pub(crate) fn update_edit_filter(
        ui: &mut Ui,
        ui_theme: &UITheme,
        filter_data: &mut EditFilterData
    ) -> OverlayAction {

        OverlayHelper::helper_update_header(ui, ui_theme, "Edit Filter");

        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Main Content", true, |ui| {
            OverlayHelper::helper_update_desc(ui, ui_theme, &mut filter_data.filter.operation, false);
        });

        OverlayHelper::helper_update_small_spacer(ui, ui_theme);
        OverlayHelper::helper_update_errors(ui, ui_theme, &filter_data.errors);

        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Update".to_string())) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            DialogOptions::Close => OverlayAction::CloseOverlay,
            DialogOptions::Confirm => OverlayAction::EditFilter(filter_data.filter.clone()),
        }
    }

    pub(crate) fn update_delete_filter(
        ui: &mut Ui,
        ui_theme: &UITheme,
        filter_data: &mut DeleteFilterData
    ) -> OverlayAction {
        OverlayHelper::helper_update_header(ui, ui_theme, "Delete Filter");

        OverlayHelper::helper_update_warning(ui, ui_theme, 
            format!("Are you absolutely sure, that you want to delete the Filter\n\"{}\"\nfrom the Adapter\n\"{}\"?", 
            filter_data.filter.identifier.name, filter_data.filter.identifier.adapter).as_str());

        OverlayHelper::helper_update_small_spacer(ui, ui_theme);
        OverlayHelper::helper_update_errors(ui, ui_theme, &filter_data.errors);

        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Delete".to_string())) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            DialogOptions::Close => OverlayAction::CloseOverlay,
            DialogOptions::Confirm => OverlayAction::DeleteFilter(filter_data.filter.clone()),
        }
    }
}

impl OverlayAction {
    pub(crate) fn action_filter(
        ui_controller: &mut UIController,
        cache: &mut UICache,
        filter: Filter
    ) {
        let mut action_successful: bool = false;
        ui_controller.using_ticket_provider_mut(|controller, provider| {
            match provider.filter_validate(&filter) {
                Ok(_) => {
                    match provider.filter_write(&filter) {
                        Ok(_) => action_successful = true,
                        Err(error) => {

                            let error_message = error.get_text();
                            let mut errors = vec![("other".to_string(), error_message)];

                            Overlay::put_errors(controller.get_current_overlay(), &mut errors);

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

        if action_successful {
            ui_controller.close_overlay();
            ui_controller.update_bucket_panel_data();
            ui_controller.execute_bucket_panel_selection();
        }
    }

    pub(crate) fn action_filter_delete(
        ui_controller: &mut UIController,
        cache: &mut UICache,
        filter: Filter
    ) {
        let mut action_successful: bool = false;
        ui_controller.using_ticket_provider_mut(|controller, provider| {

            match provider.filter_drop(&filter) {
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
            ui_controller.update_bucket_panel_data();
            ui_controller.execute_bucket_panel_selection();
        }
    }
}