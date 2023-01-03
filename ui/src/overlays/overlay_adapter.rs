use egui::Ui;

use crate::{UIController, UITheme, UICache};

use super::{OverlayAction, Overlay, helper::OverlayHelper, DialogOptions};

#[derive(Default, PartialEq, Clone)]
pub struct DeleteAdapterData {
    pub adapter_name: String
}


impl Overlay {
    pub(crate) fn update_delete_adapter(
        ui: &mut Ui,
        ui_theme: &mut UITheme,
        adapter_data: &mut DeleteAdapterData
    ) -> OverlayAction {
        OverlayHelper::helper_update_header(ui, ui_theme, "Delete Adapter");

        OverlayHelper::helper_update_warning(ui, ui_theme, 
            format!("Are you absolutely sure, that you want to remove the adapter
            \"{}\" from the Program?\nThis will only remove the reference, additional Data might need to be removed by Hand.", 
            adapter_data.adapter_name).as_str());

        OverlayHelper::helper_update_small_spacer(ui, ui_theme);

        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Delete".to_string())) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            DialogOptions::Close => OverlayAction::CloseOverlay,
            DialogOptions::Confirm => OverlayAction::DeleteAdapter(adapter_data.adapter_name.clone()),
        }
    }
}

impl OverlayAction {
    pub(crate) fn action_adapter_delete(ui_controller: &mut UIController, adapter_name: String) {
        ui_controller.close_overlay();
        ui_controller.using_ticket_provider(|_, provider| {
             provider.drop_adapter(adapter_name.clone(), true);
        });
        ui_controller.update_bucket_panel_data();
    }
}