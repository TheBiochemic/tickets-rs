use eframe::egui::Ui;
use tickets_rs_core::State;

use crate::{Overlay, UITheme, UIController, UICache};

use super::{OverlayAction, helper::OverlayHelper, DialogOptions};



#[derive(Default, PartialEq, Clone)]
pub struct NewStateData {
    pub state: State,
    pub adapters: Vec<(String, String)>,
    pub errors: Vec<(String, String)>,
}

impl Overlay {

    pub(crate) fn update_new_state(
        ui: &mut Ui,
        ui_theme: &mut UITheme,
        state_data: &mut NewStateData
    ) -> OverlayAction {

        OverlayHelper::helper_update_header(ui, ui_theme, "New State");
        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Location & Sorting", true, |ui| {
            OverlayHelper::helper_update_adapter(ui, ui_theme, &mut state_data.state.identifier.adapter, &state_data.adapters);
        });

        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Main Content", true, |ui| {
            OverlayHelper::helper_update_text(ui, ui_theme, &mut state_data.state.identifier.name, "Name:");
            OverlayHelper::helper_update_small_spacer(ui, ui_theme);
            OverlayHelper::helper_update_text(ui, ui_theme, &mut state_data.state.description, "Description:");
        });

        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Additional", false, |ui| {
            OverlayHelper::helper_update_number64(ui, ui_theme, &mut state_data.state.sorting_order, "Sorting Order");
        });

        OverlayHelper::helper_update_small_spacer(ui, ui_theme);
        OverlayHelper::helper_update_errors(ui, ui_theme, &state_data.errors);
        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Create State".to_string())) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            DialogOptions::Close => OverlayAction::CloseOverlay,
            DialogOptions::Confirm => OverlayAction::NewState(state_data.state.clone()),
        }
    }
}

impl OverlayAction {
    pub(crate) fn action_state(
        ui_controller: &mut UIController,
        cache: &mut UICache,
        state: State
    ) {
        ui_controller.using_ticket_provider_mut(|controller, provider| {
            match provider.state_validate(&state) {
                Ok(_) => {
                    match provider.state_write(&state) {
                        Ok(_) => {
                            cache.states_valid = false;
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
}