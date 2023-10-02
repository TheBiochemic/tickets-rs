use eframe::egui::Ui;
use tickets_rs_core::Bucket;

use crate::{Overlay, UITheme, UIController, UICache};

use super::{OverlayAction, helper::OverlayHelper, DialogOptions};



#[derive(Default, PartialEq, Clone)]
pub struct NewBucketData {
    pub bucket: Bucket,
    pub adapters: Vec<(String, String)>,
    pub errors: Vec<(String, String)>,
}

#[derive(Default, PartialEq, Clone)]
pub struct DeleteBucketData {
    pub bucket: Bucket,
    pub errors: Vec<(String, String)>,
}

impl Overlay {

    pub(crate) fn update_new_bucket(
        ui: &mut Ui,
        ui_theme: &mut UITheme,
        bucket_data: &mut NewBucketData
    ) -> OverlayAction {

        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Main Content", true, |ui| {
            OverlayHelper::helper_update_adapter(ui, ui_theme, &mut bucket_data.bucket.identifier.adapter, &bucket_data.adapters);
            OverlayHelper::helper_update_small_spacer(ui, ui_theme);
            OverlayHelper::helper_update_text(ui, ui_theme, &mut bucket_data.bucket.name, "Name:");
        });

        OverlayHelper::helper_update_small_spacer(ui, ui_theme);
        OverlayHelper::helper_update_errors(ui, ui_theme, &bucket_data.errors);
        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Create Bucket".to_string())) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            DialogOptions::Close => OverlayAction::CloseOverlay,
            DialogOptions::Confirm => OverlayAction::NewBucket(bucket_data.bucket.clone()),
        }
    }

    pub(crate) fn update_delete_bucket(
        ui: &mut Ui, 
        ui_theme: &UITheme,
        bucket_data: &mut DeleteBucketData
    ) -> OverlayAction {
        OverlayHelper::helper_update_header(ui, ui_theme, "Delete Bucket");

        OverlayHelper::helper_update_warning(ui, ui_theme, 
            format!("Are you absolutely sure, that you want to delete the Bucket\n\"{}\"\nfrom the Adapter\n\"{}\"?", 
            bucket_data.bucket.name, bucket_data.bucket.identifier.adapter).as_str());

        OverlayHelper::helper_update_small_spacer(ui, ui_theme);
        OverlayHelper::helper_update_errors(ui, ui_theme, &bucket_data.errors);

        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Delete".to_string())) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            DialogOptions::Close => OverlayAction::CloseOverlay,
            DialogOptions::Confirm => OverlayAction::DeleteBucket(bucket_data.bucket.clone()),
        }
    }
}

impl OverlayAction {

    pub(crate) fn action_bucket_delete(
        ui_controller: &mut UIController,
        bucket: Bucket
    ) {
        let mut action_successful: bool = false;
        ui_controller.using_ticket_provider_mut(|controller, provider| {

            match provider.bucket_drop(&bucket) {
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
        }
    }

    pub(crate) fn action_bucket(
        ui_controller: &mut UIController,
        cache: &mut UICache,
        mut bucket: Bucket
    ) {
        let mut bucket_write_successful = false;

        ui_controller.using_ticket_provider_mut(|controller, provider| {
            match provider.bucket_validate(&bucket) {
                Ok(_) => {
                    match provider.bucket_write(&mut bucket) {
                        Ok(_) => bucket_write_successful = true,
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

        if bucket_write_successful {
            ui_controller.update_bucket_panel_data();
            ui_controller.close_overlay()
        }
    }
}