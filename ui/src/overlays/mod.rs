mod helper;
mod overlay_tag;
mod overlay_ticket;
mod overlay_about;
mod overlay_wizard;
mod overlay_state;
mod overlay_bucket;
mod overlay_preferences;
mod overlay_adapter;
mod overlay_filter;

use std::collections::{
    HashMap, 
    hash_map::RandomState
};
use egui::{
    Ui, 
    Color32, TextureHandle, ColorImage, Image, Vec2, RichText, Button, 
};

use chrono::{Date, Utc, TimeZone, Duration, DateTime, Datelike, Timelike, offset};
use tickets_rs_core::{State, Filter};
use tickets_rs_core::{Ticket, Bucket, Tag};

use helper::OverlayHelper as OverlayHelper;
use crate::{UITheme, UserInterface, UIController, ui_controller, UICache};

pub use self::overlay_tag::NewTagData;
pub use self::overlay_ticket::EditTicketData;
pub use self::overlay_ticket::NewTicketData;
pub use self::overlay_ticket::UpdateTicketData;
pub use self::overlay_ticket::UpdateTicketDataAssign;
pub use self::overlay_ticket::UpdateTicketDataBucket;
pub use self::overlay_ticket::UpdateTicketDataAdapter;
pub use self::overlay_wizard::WizardData;
pub use self::overlay_state::NewStateData;
pub use self::overlay_bucket::NewBucketData;
pub use self::overlay_bucket::DeleteBucketData;
pub use self::overlay_preferences::PreferenceData;
pub use self::overlay_adapter::DeleteAdapterData;
pub use self::overlay_filter::NewFilterData;
pub use self::overlay_filter::EditFilterData;
pub use self::overlay_filter::DeleteFilterData;

#[derive(PartialEq, Clone)]
pub enum Overlay {
    None,

    Wizard(WizardData),
    About,
    Preferences(PreferenceData),

    NewTicket(NewTicketData),
    EditTicket(EditTicketData),
    UpdateTicketState(UpdateTicketData),
    UpdateTicketDetails(UpdateTicketData),
    UpdateTicketBucket(UpdateTicketDataBucket),
    UpdateTicketAssign(UpdateTicketDataAssign),
    UpdateTicketAdapter(UpdateTicketDataAdapter),
    DeleteTicket(UpdateTicketData),

    NewBucket(NewBucketData),
    DeleteBucket(DeleteBucketData),

    NewState(NewStateData),

    DeleteAdapter(DeleteAdapterData),
    
    NewTag(NewTagData),

    NewFilter(NewFilterData),
    EditFilter(EditFilterData),
    DeleteFilter(DeleteFilterData)

}

#[derive(PartialEq, Clone)]
pub enum DialogOptions {
    Nothing,
    Close,
    Confirm
}

#[derive(PartialEq)]
pub enum OverlayAction {
    Nothing,
    CloseOverlay,

    NewTicket(Ticket),
    UpdateTicket(Ticket),
    UpdateTicketAdapter(Ticket, String), //Old Adapter Name
    DeleteTicket(Ticket),

    NewTag(Tag),
    UpdateTag(Tag),
    DeleteTag(Tag),

    NewState(State),
    UpdateState(State),

    NewBucket(Bucket),
    UpdateBucket(Bucket),
    DeleteBucket(Bucket),

    WizardDone(WizardData),
    PreferencesApply(PreferenceData),

    DeleteAdapter(String),

    NewFilter(Filter),
    EditFilter(Filter),
    DeleteFilter(Filter)
}

impl Overlay {

    pub fn put_errors(overlay: &mut Overlay, errors: &mut Vec<(String, String)>){
            let overlay_errors = match overlay {

                Overlay::NewTicket(ticket_data) => &mut ticket_data.errors,
                Overlay::EditTicket(ticket_data) => &mut ticket_data.errors,
                Overlay::UpdateTicketState(ticket_data) => &mut ticket_data.errors,
                Overlay::UpdateTicketDetails(ticket_data) => &mut ticket_data.errors,
                Overlay::UpdateTicketBucket(ticket_data) => &mut ticket_data.errors,
                Overlay::UpdateTicketAssign(ticket_data) => &mut ticket_data.errors,
                Overlay::UpdateTicketAdapter(ticket_data) => &mut ticket_data.errors,
                Overlay::DeleteTicket(ticket_data) => &mut ticket_data.errors,
                Overlay::NewTag(tag_data) => &mut tag_data.errors,
                Overlay::NewState(state_data) => &mut state_data.errors,
                Overlay::NewBucket(bucket_data) => &mut bucket_data.errors,
                Overlay::NewFilter(filter_data) => &mut filter_data.errors,
                Overlay::EditFilter(filter_data) => &mut filter_data.errors,
                Overlay::DeleteFilter(filter_data) => &mut filter_data.errors,
                _ => return
            };

            overlay_errors.clear();
            overlay_errors.append(errors)

    }

    pub fn update(
        overlay: &mut Overlay, 
        ui: &mut Ui, 
        ui_theme: &mut UITheme, 
        ui_controller: &mut UIController,
        cache: &mut UICache,
        icon_textures: &mut HashMap<String, Option<TextureHandle>, RandomState>,
        icons: &mut HashMap<String, Option<ColorImage>, RandomState>,
        
    ) -> OverlayAction {
        match overlay {

            Overlay::None => OverlayAction::Nothing,
            Overlay::Wizard(wizard_data) => Overlay::update_wizard(ui, ui_theme, ui_controller, wizard_data),
            Overlay::Preferences(preference_data) => Overlay::update_preferences(ui, ui_theme, ui_controller, preference_data),
            Overlay::NewBucket(bucket_data) => Overlay::update_new_bucket(ui, ui_theme, bucket_data),
            Overlay::NewTicket(ticket_data) => Overlay::update_new_ticket(ui, ui_theme, ticket_data, cache),
            Overlay::NewState(state_data) => Overlay::update_new_state(ui, ui_theme, state_data),
            Overlay::UpdateTicketState(ticket_data) => Overlay::update_ticket_state(ui, ui_theme, cache, ticket_data),
            Overlay::UpdateTicketDetails(ticket_data) => Overlay::update_ticket_details(ui, ui_theme, ticket_data),
            Overlay::UpdateTicketBucket(ticket_data) => Overlay::update_ticket_bucket(ui, ui_theme, ticket_data),
            Overlay::UpdateTicketAssign(ticket_data) => Overlay::update_ticket_assign(ui, ui_theme, ticket_data),
            Overlay::UpdateTicketAdapter(ticket_data) => Overlay::update_ticket_adapter(ui, ui_theme, ticket_data),
            Overlay::DeleteTicket(ticket_data) => Overlay::update_delete_ticket(ui, ui_theme, ticket_data),
            Overlay::About =>                   Overlay::update_about(ui, ui_theme, icon_textures, icons),
            Overlay::NewTag(tag_data) =>        Overlay::update_new_tag(ui, ui_theme, tag_data),
            Overlay::EditTicket(ticket_data) => Overlay::update_edit_ticket(ui, ui_theme, ticket_data, cache),
            Overlay::DeleteAdapter(adapter_data) => Overlay::update_delete_adapter(ui, ui_theme, adapter_data),
            Overlay::NewFilter(filter_data) => Overlay::update_new_filter(ui, ui_theme, filter_data),
            Overlay::EditFilter(filter_data) => Overlay::update_edit_filter(ui, ui_theme, filter_data),
            Overlay::DeleteFilter(filter_data) => Overlay::update_delete_filter(ui, ui_theme, filter_data),
            Overlay::DeleteBucket(bucket_data) => Overlay::update_delete_bucket(ui, ui_theme, bucket_data),

        }
    }
}

impl OverlayAction {
    pub fn execute(self, ui_controller: &mut UIController, cache: &mut UICache) {
        match self {
            OverlayAction::UpdateTicket(ticket) => OverlayAction::action_ticket(ui_controller, ticket),
            OverlayAction::NewTicket(ticket) => OverlayAction::action_ticket(ui_controller, ticket),
            OverlayAction::CloseOverlay => ui_controller.close_overlay(),
            OverlayAction::Nothing => (),
            OverlayAction::NewState(state) => OverlayAction::action_state(ui_controller, cache, state),
            OverlayAction::NewTag(tag) => OverlayAction::action_tag(ui_controller, cache, tag),
            OverlayAction::PreferencesApply(preference_data) => OverlayAction::action_preferences(ui_controller, cache, preference_data),
            OverlayAction::WizardDone(wizard_data) => OverlayAction::action_wizard(ui_controller, cache, wizard_data),
            OverlayAction::UpdateTag(tag) => OverlayAction::action_tag(ui_controller, cache, tag),
            OverlayAction::UpdateState(state) => OverlayAction::action_state(ui_controller, cache, state),
            OverlayAction::NewBucket(bucket) => OverlayAction::action_bucket(ui_controller, cache, bucket),
            OverlayAction::UpdateBucket(bucket) => OverlayAction::action_bucket(ui_controller, cache, bucket),
            OverlayAction::DeleteTicket(ticket) => OverlayAction::action_ticket_delete(ui_controller, ticket),
            OverlayAction::DeleteTag(tag) => OverlayAction::action_tag_delete(ui_controller, cache, tag),
            OverlayAction::DeleteAdapter(adapter_name) => OverlayAction::action_adapter_delete(ui_controller, adapter_name),
            OverlayAction::UpdateTicketAdapter(ticket, old_adapter_name) => OverlayAction::action_ticket_adapter(ui_controller, ticket, old_adapter_name),
            OverlayAction::NewFilter(filter) => OverlayAction::action_filter(ui_controller, cache, filter),
            OverlayAction::EditFilter(filter) => OverlayAction::action_filter(ui_controller, cache, filter),
            OverlayAction::DeleteFilter(filter) => OverlayAction::action_filter_delete(ui_controller, cache, filter),
            OverlayAction::DeleteBucket(bucket) => OverlayAction::action_bucket_delete(ui_controller, bucket),
        };
    }
}