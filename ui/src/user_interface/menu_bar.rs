use egui::{
    Rounding, 
    style::Margin, 
    Stroke, 
    Frame, 
    Ui,
    menu
};

use crate::{
    UserInterface, 
    ui_controller::TicketViewMode, 
    Overlay, 
    overlays::WizardData
};


impl UserInterface {

    pub fn update_menu_bar(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        let frame = Frame {
            inner_margin: Margin::same(8.0),
            outer_margin: Margin::same(0.0),
            rounding: Rounding::same(0.0),
            shadow: ctx.style().visuals.popup_shadow,
            fill: ctx.style().visuals.window_fill(),
            stroke: Stroke::none(),
        };

        menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {

                // Setup the menu internals first
                let button_new_ticket = ui.button("New Ticket")
                    .on_hover_text_at_pointer("Shows a Dialog Window for creating a new Ticket");
                
                let button_import_ticket = ui.button("Import Ticket")
                    .on_hover_text_at_pointer("Imports a previously exported Ticket into tickets.rs");

                let button_add_tickets = ui.button("Add Tickets...")
                    .on_hover_text_at_pointer("Shows Dialog Window for adding Tickets in bulk");

                ui.separator();

                let button_new_bucket = ui.button("New Bucket")
                    .on_hover_text_at_pointer("Shows a Dialog Window for creating a new Bucket");

                let button_import_bucket = ui.button("Import Bucket")
                    .on_hover_text_at_pointer("Imports a previously exported Bucket into tickets.rs");

                ui.separator();

                let button_exit = ui.button("Exit")
                    .on_hover_text_at_pointer("Exits tickets.rs");

                // Events Handling
                if button_new_ticket.clicked() {
                    self.ui_controller.open_overlay(self.ui_controller.create_new_ticket_overlay(None));
                    ui.close_menu();
                }

                if button_import_ticket.clicked() {
                    ui.close_menu();
                }

                if button_add_tickets.clicked() {
                    ui.close_menu();
                }

                if button_new_bucket.clicked() {
                    self.ui_controller.open_overlay(self.ui_controller.create_new_bucket_overlay(None));
                    ui.close_menu();
                }

                if button_import_bucket.clicked() {
                    ui.close_menu();
                }

                if button_exit.clicked() {
                    self.ui_controller.running = false;
                    ui.close_menu();
                }

            });

            ui.menu_button("Edit", |ui| {

                // Setup the menu points itself
                let button_refresh = ui.button("Refresh")
                    .on_hover_text_at_pointer("Pulls in the currently displayed data in again.");

                let button_preferences = ui.button("Preferences...")
                    .on_hover_text_at_pointer("Shows a Dialog Window changing settings regarding tickets.rs");

                // Handle the Events
                if button_refresh.clicked() {
                    self.ui_controller.update_bucket_panel_data();
                    ui.close_menu();
                }

                if button_preferences.clicked() {
                    self.ui_controller.open_overlay(self.ui_controller.create_preferences_overlay());
                    ui.close_menu();
                }
            });

            ui.menu_button("View", |ui| {

                let choice_regular = ui.radio_value(&mut self.ui_controller.ticket_view_mode, TicketViewMode::Regular, "Display Tickets full size")
                    .on_hover_text_at_pointer("Do the tickets need to be displayed in full size? If yes, then click this option.");

                let choice_half = ui.radio_value(&mut self.ui_controller.ticket_view_mode, TicketViewMode::Half, "Display Tickets half size")
                    .on_hover_text_at_pointer("If you want to see more Tickets at once, but don't want to give up the Description, this setting is for you.");

                let choice_list = ui.radio_value(&mut self.ui_controller.ticket_view_mode, TicketViewMode::List, "Display Tickets as list")
                    .on_hover_text_at_pointer("For a quick overview, you can switch the ticket display to a compact list.");

                ui.separator();

                let checkbox_sidebar = ui.checkbox(&mut self.ui_controller.show_sidebar, "Show Sidebar")
                    .on_hover_text_at_pointer("Toggles, if the Sidebar, that shows Filters and Adapters is hidden, or not.");


                if choice_regular.clicked() ||
                   choice_half.clicked() ||
                   choice_list.clicked() {

                    match self.ui_controller.configuration.lock(){
                        Ok(mut lock) => {
                            match self.ui_controller.ticket_view_mode {
                                TicketViewMode::Regular => lock.put("ticket:view_mode", "regular", ""),
                                TicketViewMode::Half => lock.put("ticket:view_mode", "half", ""),
                                TicketViewMode::List => lock.put("ticket:view_mode", "list", ""),
                            }
                        },
                        Err(err) => println!("Wasn't able to lock Config, when pushing a view Button, due to {err}"),
                    }
                    

                    ui.close_menu();
                };

                if checkbox_sidebar.clicked() {

                    match self.ui_controller.configuration.lock(){
                        Ok(mut lock) => {
                            lock.put("sidebar:enabled", self.ui_controller.show_sidebar, "");
                        },
                        Err(err) => println!("Wasn't able to lock Config, when pushing a view Button, due to {err}"),
                    }

                    ui.close_menu();
                }
            });
        

            ui.menu_button("Help", |ui| {

                let button_wizard = ui.button("Reopen Wizard...")
                    .on_hover_text_at_pointer("Just in case you accidentally closed it, you can reopen the Wizard here again.");

                let button_about = ui.button("About tickets.rs")
                    .on_hover_text_at_pointer("Show some information about the Program and it's contributors.");

                if button_about.clicked() {
                    self.ui_controller.open_overlay(Overlay::About);
                    ui.close_menu();
                }

                if button_wizard.clicked() {

                    let mut username = "New User".to_string();
                    match self.ui_controller.configuration.lock() {
                        Ok(mut lock) => username = lock.get_or_default("username", "New User", "").raw().clone(),
                        Err(err) => println!("Wasn't able to lock App Config for Wizard due to {err}"),
                    }

                    self.ui_controller.open_overlay(Overlay::Wizard(WizardData{
                        username,
                        ..Default::default()
                    }));
                    ui.close_menu();
                }
            });
        });
    }

}