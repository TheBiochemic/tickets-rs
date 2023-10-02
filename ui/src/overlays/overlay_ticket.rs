use std::collections::{hash_map::RandomState, HashMap};

use chrono::{Utc, DateTime};
use eframe::egui::{Ui, Color32};
use tickets_rs_core::{Bucket, Ticket};

use crate::{Overlay, UITheme, UIController, UICache};

use super::{OverlayHelper, OverlayAction, DialogOptions};


#[derive(Default, PartialEq, Clone)]
pub struct NewTicketData {
    pub ticket: Ticket,
    pub tag_text: String,
    pub assigned_text: String,
    pub due_date: DateTime<Utc>,
    pub username: String,
    pub buckets: Vec<Bucket>,
    pub adapters: Vec<(String, String)>,
    pub errors: Vec<(String, String)>,
}

#[derive(Default, PartialEq, Clone)]
pub struct EditTicketData {
    pub ticket: Ticket,
    pub tag_text: String,
    pub assigned_text: String,
    pub due_date: DateTime<Utc>,
    pub username: String,
    pub buckets: Vec<Bucket>,
    pub errors: Vec<(String, String)>,
}

#[derive(Default, PartialEq, Clone)]
pub struct UpdateTicketData {
    pub ticket: Ticket,
    pub errors: Vec<(String, String)>,
}

#[derive(Default, PartialEq, Clone)]
pub struct UpdateTicketDataBucket {
    pub ticket: Ticket,
    pub buckets: Vec<Bucket>,
    pub errors: Vec<(String, String)>,
}

#[derive(Default, PartialEq, Clone)]
pub struct UpdateTicketDataAssign {
    pub ticket: Ticket,
    pub username: String,
    pub assigned_text: String,
    pub errors: Vec<(String, String)>,
}

#[derive(Default, PartialEq, Clone)]
pub struct UpdateTicketDataAdapter {
    pub ticket: Ticket,
    pub old_adapter: String,
    pub adapters: Vec<(String, String)>,
    pub errors: Vec<(String, String)>,
}

impl Overlay {

    pub(crate) fn update_ticket_adapter(
        ui: &mut Ui,
        ui_theme: &UITheme,
        ticket_data: &mut UpdateTicketDataAdapter
    ) -> OverlayAction {

        OverlayHelper::helper_update_header(ui, ui_theme, "New Ticket");
        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Location & Sorting", true, |ui| {
            OverlayHelper::helper_update_adapter(ui, ui_theme, &mut ticket_data.ticket.adapter, &ticket_data.adapters);
        });

        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Move".to_string())) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            DialogOptions::Close => OverlayAction::CloseOverlay,
            DialogOptions::Confirm => OverlayAction::UpdateTicketAdapter(ticket_data.ticket.clone(), ticket_data.old_adapter.clone()),
        }
    }

    pub(crate) fn update_ticket_state(
        ui: &mut Ui, 
        ui_theme: &UITheme,
        cache: &mut UICache,
        ticket_data: &mut UpdateTicketData
    ) -> OverlayAction {
        OverlayHelper::helper_update_header(ui, ui_theme, "Update State");
        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Location & Sorting", true, |ui| {
            OverlayHelper::helper_update_state(ui, ui_theme, &mut ticket_data.ticket.state_name, &cache.states, &ticket_data.ticket.adapter);
        });

        OverlayHelper::helper_update_small_spacer(ui, ui_theme);
        OverlayHelper::helper_update_errors(ui, ui_theme, &ticket_data.errors);

        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Update".to_string())) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            DialogOptions::Close => OverlayAction::CloseOverlay,
            DialogOptions::Confirm => OverlayAction::UpdateTicket(ticket_data.ticket.clone()),
        }
    }

    pub(crate) fn update_ticket_assign(
        ui: &mut Ui, 
        ui_theme: &UITheme,
        ticket_data: &mut UpdateTicketDataAssign
    ) -> OverlayAction {
        OverlayHelper::helper_update_header(ui, ui_theme, "Assign to");
        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Additional", true, |ui| {
            OverlayHelper::helper_update_assigned(ui, ui_theme, &mut ticket_data.ticket.assigned_to, &mut ticket_data.assigned_text, &ticket_data.username);
        });

        OverlayHelper::helper_update_small_spacer(ui, ui_theme);
        OverlayHelper::helper_update_errors(ui, ui_theme, &ticket_data.errors);

        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Assign".to_string())) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            DialogOptions::Close => OverlayAction::CloseOverlay,
            DialogOptions::Confirm => OverlayAction::UpdateTicket(ticket_data.ticket.clone()),
        }
    }

    pub(crate) fn update_ticket_details(
        ui: &mut Ui, 
        ui_theme: &UITheme,
        ticket_data: &mut UpdateTicketData
    ) -> OverlayAction {

        OverlayHelper::helper_update_header(ui, ui_theme, "Update Details");

        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Main Content", true, |ui| {
            OverlayHelper::helper_update_text(ui, ui_theme, &mut ticket_data.ticket.title, "Name:");
            OverlayHelper::helper_update_small_spacer(ui, ui_theme);
            OverlayHelper::helper_update_desc(ui, ui_theme, &mut ticket_data.ticket.description, true);
        });

        OverlayHelper::helper_update_small_spacer(ui, ui_theme);
        OverlayHelper::helper_update_errors(ui, ui_theme, &ticket_data.errors);

        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Update".to_string())) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            DialogOptions::Close => OverlayAction::CloseOverlay,
            DialogOptions::Confirm => OverlayAction::UpdateTicket(ticket_data.ticket.clone()),
        }
    }

    pub(crate) fn update_ticket_bucket(
        ui: &mut Ui, 
        ui_theme: &UITheme,
        ticket_data: &mut UpdateTicketDataBucket
    ) -> OverlayAction {

        OverlayHelper::helper_update_header(ui, ui_theme, "Move to Bucket");

        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Location & Sorting", true, |ui| {
            OverlayHelper::helper_update_bucket(ui, ui_theme, &mut ticket_data.ticket.bucket_id, &ticket_data.buckets, &ticket_data.ticket.adapter);
        });

        OverlayHelper::helper_update_small_spacer(ui, ui_theme);
        OverlayHelper::helper_update_errors(ui, ui_theme, &ticket_data.errors);

        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Move".to_string())) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            DialogOptions::Close => OverlayAction::CloseOverlay,
            DialogOptions::Confirm => OverlayAction::UpdateTicket(ticket_data.ticket.clone()),
        }
    }

    pub(crate) fn update_delete_ticket(
        ui: &mut Ui, 
        ui_theme: &UITheme,
        ticket_data: &mut UpdateTicketData
    ) -> OverlayAction {
        OverlayHelper::helper_update_header(ui, ui_theme, "Delete Ticket");

        OverlayHelper::helper_update_warning(ui, ui_theme, 
            format!("Are you absolutely sure, that you want to delete the Ticket\n\"{}\"\nfrom the Adapter\n\"{}\"?", 
            ticket_data.ticket.title, ticket_data.ticket.adapter).as_str());

        OverlayHelper::helper_update_small_spacer(ui, ui_theme);
        OverlayHelper::helper_update_errors(ui, ui_theme, &ticket_data.errors);

        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Delete".to_string())) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            DialogOptions::Close => OverlayAction::CloseOverlay,
            DialogOptions::Confirm => OverlayAction::DeleteTicket(ticket_data.ticket.clone()),
        }
    }

    pub(crate) fn update_new_ticket(
        ui: &mut Ui, 
        ui_theme: &UITheme, 
        ticket_data: &mut NewTicketData, 
        cache: &mut UICache,
    ) -> OverlayAction {
        
        OverlayHelper::helper_update_header(ui, ui_theme, "New Ticket");
        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Location & Sorting", true, |ui| {
            OverlayHelper::helper_update_adapter(ui, ui_theme, &mut ticket_data.ticket.adapter, &ticket_data.adapters);
            OverlayHelper::helper_update_small_spacer(ui, ui_theme);
            OverlayHelper::helper_update_bucket(ui, ui_theme, &mut ticket_data.ticket.bucket_id, &ticket_data.buckets, &ticket_data.ticket.adapter);
            OverlayHelper::helper_update_small_spacer(ui, ui_theme);
            OverlayHelper::helper_update_state(ui, ui_theme, &mut ticket_data.ticket.state_name, &cache.states, &ticket_data.ticket.adapter);
        });
    
        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Main Content", true, |ui| {
            OverlayHelper::helper_update_text(ui, ui_theme, &mut ticket_data.ticket.title, "Name:");
            OverlayHelper::helper_update_small_spacer(ui, ui_theme);
            OverlayHelper::helper_update_desc(ui, ui_theme, &mut ticket_data.ticket.description, true);
        });
    
        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Additional", false, |ui| {
            
            OverlayHelper::helper_update_tags(
                ui, 
                ui_theme, 
                &mut ticket_data.ticket.tags, 
                &mut ticket_data.tag_text, 
                &ticket_data.ticket.adapter,
                &cache.tags
            );

            OverlayHelper::helper_update_small_spacer(ui, ui_theme);
            OverlayHelper::helper_update_assigned(ui, ui_theme, &mut ticket_data.ticket.assigned_to, &mut ticket_data.assigned_text, &ticket_data.username);
            OverlayHelper::helper_update_small_spacer(ui, ui_theme);
            if let Some(new_ts) = OverlayHelper::helper_update_due(ui, ui_theme, &mut ticket_data.due_date) {
                ticket_data.ticket.due_at = new_ts;
            }
        });
    
        OverlayHelper::helper_update_small_spacer(ui, ui_theme);
        OverlayHelper::helper_update_errors(ui, ui_theme, &ticket_data.errors);
        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Create".to_string())) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            DialogOptions::Close => OverlayAction::CloseOverlay,
            DialogOptions::Confirm => OverlayAction::NewTicket(ticket_data.ticket.clone()),
        }
    }

    pub(crate) fn update_edit_ticket(
        ui: &mut Ui, 
        ui_theme: &UITheme, 
        ticket_data: &mut EditTicketData, 
        cache: &mut UICache,
    ) -> OverlayAction {
        
        OverlayHelper::helper_update_header(ui, ui_theme, "Edit Ticket");
        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Location & Sorting", true, |ui| {
            OverlayHelper::helper_update_bucket(ui, ui_theme, &mut ticket_data.ticket.bucket_id, &ticket_data.buckets, &ticket_data.ticket.adapter);
            OverlayHelper::helper_update_small_spacer(ui, ui_theme);
            OverlayHelper::helper_update_state(ui, ui_theme, &mut ticket_data.ticket.state_name, &cache.states, &ticket_data.ticket.adapter);
        });
    
        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Main Content", true, |ui| {
            OverlayHelper::helper_update_text(ui, ui_theme, &mut ticket_data.ticket.title, "Name:");
            OverlayHelper::helper_update_small_spacer(ui, ui_theme);
            OverlayHelper::helper_update_desc(ui, ui_theme, &mut ticket_data.ticket.description, true);
        });
    
        OverlayHelper::helper_update_section_collapsing(ui, ui_theme, "Additional", false, |ui| {
            
            OverlayHelper::helper_update_tags(
                ui, 
                ui_theme, 
                &mut ticket_data.ticket.tags, 
                &mut ticket_data.tag_text, 
                &ticket_data.ticket.adapter,
                &cache.tags
            );

            OverlayHelper::helper_update_small_spacer(ui, ui_theme);
            OverlayHelper::helper_update_assigned(ui, ui_theme, &mut ticket_data.ticket.assigned_to, &mut ticket_data.assigned_text, &ticket_data.username);
            OverlayHelper::helper_update_small_spacer(ui, ui_theme);
            if let Some(new_ts) = OverlayHelper::helper_update_due(ui, ui_theme, &mut ticket_data.due_date) {
                ticket_data.ticket.due_at = new_ts;
            }
        });
    
        OverlayHelper::helper_update_small_spacer(ui, ui_theme);
        OverlayHelper::helper_update_errors(ui, ui_theme, &ticket_data.errors);
        match OverlayHelper::helper_update_dialog_buttons(ui, ui_theme, Some("Edit".to_string())) {
            DialogOptions::Nothing => OverlayAction::Nothing,
            DialogOptions::Close => OverlayAction::CloseOverlay,
            DialogOptions::Confirm => OverlayAction::UpdateTicket(ticket_data.ticket.clone()),
        }
    }
}

impl OverlayAction {

    pub(crate) fn action_ticket_adapter(
        ui_controller: &mut UIController,
        ticket: Ticket,
        old_adapter: String
    ) {

        //if the old adapter and the new one are the same, dont do anything
        if old_adapter == ticket.adapter {
            ui_controller.close_overlay();
            return;
        }

        let mut create_successful = false;
        let mut delete_successful = false;

        ui_controller.using_ticket_provider_mut(|controller, provider| {
            let mut ticket_clone = ticket.clone();
            ticket_clone.id = 0;

            //Create the ticket on the new adapter first
            match provider.ticket_validate(&ticket_clone) {
                Ok(_) => {
                    match provider.ticket_write(&ticket_clone) {
                        Ok(_) => create_successful = true,
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

            //If everything went fine, delete the old ticket from old adapter
            if create_successful {

                let mut ticket_clone = ticket.clone();
                ticket_clone.adapter = old_adapter;

                match provider.ticket_drop(&ticket_clone) {
                    Ok(_) => delete_successful = true,
                    Err(error) => {
    
                        let error_message = error.get_text();
                        let mut errors = vec![("other".to_string(), error_message)];
    
                        Overlay::put_errors(controller.get_current_overlay(), &mut errors);
    
                    },
                };
            }
        });

        if create_successful && delete_successful {
            ui_controller.close_overlay();
            ui_controller.execute_bucket_panel_selection();
        }
    }

    pub(crate) fn action_ticket_delete(
        ui_controller: &mut UIController,
        ticket: Ticket
    ) {
        let mut action_successful: bool = false;
        ui_controller.using_ticket_provider_mut(|controller, provider| {

            match provider.ticket_drop(&ticket) {
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
            ui_controller.execute_bucket_panel_selection();
        }
    }

    pub(crate) fn action_ticket(
        ui_controller: &mut UIController,
        ticket: Ticket
    ) {
        let mut action_successful: bool = false;
        ui_controller.using_ticket_provider_mut(|controller, provider| {
            match provider.ticket_validate(&ticket) {
                Ok(_) => {
                    match provider.ticket_write(&ticket) {
                        Ok(_) => action_successful = true,
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

        if action_successful {
            ui_controller.close_overlay();
            ui_controller.execute_bucket_panel_selection();
        }
    }
}