use egui::{
    Ui, 
    SelectableLabel, 
    Align, Label, 
    RichText, 
    Button, 
    ScrollArea};
use tickets_rs_core::FilterType;

use crate::{
    UserInterface, 
    UITheme, 
    ui_controller::{
        BucketPanelFolder, 
        BucketPanelEntry
    }
};

pub enum SidePanelAction {
    FolderOpenClose,
    FolderClicked,
    FolderNewTag,
    FolderNewBucket,
    FolderNewFilter,
    FolderNewTicket,
    FolderNewState,
    FolderRemove,
    EntryClicked,
    EntryRemove,
    EntryEdit,
    EntryBucketRemove(i64),
    Nothing,
}

impl UserInterface {

    pub(crate) fn update_side_panel_folder(ui: &mut Ui, ui_theme: &UITheme, is_selected: bool, is_open: bool, selectable: bool, folder: &BucketPanelFolder) -> SidePanelAction {

        let mut action: SidePanelAction = SidePanelAction::Nothing;

        ui.with_layout(egui::Layout::left_to_right(Align::LEFT), |ui| {

            if ui.add_sized([16.0, 16.0], SelectableLabel::new(false, if is_open {"⊟"} else {"⊞"})).clicked() {
                action = SidePanelAction::FolderOpenClose;
            }



            if selectable {
                ui.add_sized([4.0, 16.0], Label::new(""));
                let button = SelectableLabel::new(is_selected, RichText::new(&folder.label).color(ui_theme.foreground_marker2));
                let mut response = ui.add(button);

                if response.clicked() {
                    action = SidePanelAction::FolderClicked;
                };

                response.context_menu(|ui| {

                    if ui.button("Add Ticket").clicked() {
                        ui.close_menu();
                        action = SidePanelAction::FolderNewTicket;
                    };

                    if ui.button("Add Bucket").clicked() {
                        ui.close_menu();
                        action = SidePanelAction::FolderNewBucket;
                    };

                    if ui.button("Add Tag").clicked() {
                        ui.close_menu();
                        action = SidePanelAction::FolderNewTag;
                    };

                    if ui.button("Add State").clicked() {
                        ui.close_menu();
                        action = SidePanelAction::FolderNewState;
                    };

                    if ui.button("Add Custom Filter").clicked() {
                        ui.close_menu();
                        action = SidePanelAction::FolderNewFilter
                    };

                    ui.separator();

                    if ui.button(RichText::new("Remove this Adapter").color(ui_theme.foreground_marker2)).clicked() {
                        ui.close_menu();
                        action = SidePanelAction::FolderRemove
                    };
                });

            } else {
                ui.add_sized([8.0, 16.0], Label::new(""));
                let label = Label::new(RichText::new(&folder.label).color(ui_theme.foreground_marker));
                
                let response = ui.add(label);

                if response.clicked() {
                    action = SidePanelAction::FolderClicked;
                };

                response.context_menu(|ui| {

                    if ui.button("Add Custom Filter").clicked() {
                        ui.close_menu();
                        action = SidePanelAction::FolderNewFilter
                    };

                });

                
            };

            action

        }).inner
    }

    pub(crate) fn update_side_panel_entry(ui: &mut Ui, ui_theme: &UITheme, is_selected: bool, entry: &BucketPanelEntry) -> SidePanelAction {

        let mut action = SidePanelAction::Nothing;

        let entry_icon = match entry.entry_type {
            FilterType::User => "⚙",
            FilterType::Builtin => "🔨",
            FilterType::Bucket(_) => "🗄",
            FilterType::Tag => "🏷",
            FilterType::Other => "❔",
        };

        ui.with_layout(egui::Layout::left_to_right(Align::LEFT), |ui| {

            ui.add_sized([24.0, 16.0], Label::new(""));
            ui.add_sized([16.0, 16.0], Label::new(entry_icon));
    
            let button = SelectableLabel::new(is_selected, RichText::new(&entry.label).color(ui_theme.foreground_secondary).italics());
            let mut response = ui.add(button);

            match entry.entry_type {
                FilterType::User => {
                    response = response.context_menu(|ui| {
                        if ui.button("Edit").clicked() {
                            action = SidePanelAction::EntryEdit;
                            ui.close_menu();
                        };
    
                        if ui.button(RichText::new("Remove").color(ui_theme.foreground_marker2)).clicked() {
                            action = SidePanelAction::EntryRemove;
                            ui.close_menu();
                        };
                    });
                },
                FilterType::Bucket(id) => {
                    response = response.context_menu(|ui| {
    
                        if ui.button(RichText::new("Remove").color(ui_theme.foreground_marker2)).clicked() {
                            action = SidePanelAction::EntryBucketRemove(id);
                            ui.close_menu();
                        };
                    });
                },
                _ => ()
                
            }
            

            if response.clicked() {
                action = SidePanelAction::EntryClicked
            }
        });

        action
    }

    pub(crate) fn update_side_panel_space(ui: &mut Ui) -> bool {
        let space = Button::new("").frame(false);
        ui.add_sized([ui.available_width(), ui.available_height()], space).double_clicked()
    }

    pub(crate) fn update_side_panel(&mut self, ctx: &egui::Context, ui: &mut Ui) {

        let shift_or_ctrl = ctx.input().modifiers.shift || ctx.input().modifiers.ctrl;
        let controller = &mut self.ui_controller;
        let ui_theme = &self.ui_theme;
        ScrollArea::vertical().show(ui, |ui| {

            ui.spacing_mut().item_spacing.x = 0.0;

            match controller.update_each_folder(ui, ui_theme) {
                Some(folder) => {
                    controller.toggle_folder_in_panel(folder, shift_or_ctrl);
                    controller.execute_bucket_panel_selection();
                },
                None => (),
            };
        });
    }

}