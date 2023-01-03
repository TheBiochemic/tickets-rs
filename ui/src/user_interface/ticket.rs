use egui::{
    Frame, 
    Response, 
    Ui, 
    TextureHandle, 
    Vec2, 
    Image, 
    RichText, 
    Button, 
    ScrollArea, 
    style::Margin
};

use egui_commonmark::{
    CommonMarkCache, 
    CommonMarkViewer
};

use tickets_rs_core::{Ticket, Tag, StateIdentifier};

use crate::{
    UserInterface, 
    ui_controller::ticket_actions::{
        TicketAction, 
        Identifier
    }, 
    UITheme, UICache, TagCacheKey
};

impl UserInterface {

    pub fn update_ticket_context_menu(
        response: Response,
        action: &mut TicketAction,
        ui: &mut Ui, 
        ticket: &Ticket, 
        theme: &UITheme, 
        cache: &UICache
    ) {
        let font_size = theme.font_size as f32;

        response.context_menu(|ui| {

            if ui.button("Edit Details...")
                .on_hover_text_at_pointer("Edit the title and the description of the right-clicked Ticket. Note that the description supports some sort of markdown")
                .clicked() {
                    ui.close_menu();
                    *action = TicketAction::UpdateDetails(Identifier::new(&ticket.adapter, ticket.id));
                };

            if ui.button("Assign to...")
                .on_hover_text_at_pointer("Edit the assigned participants for the right-clicked Ticket.")
                .clicked() {
                    ui.close_menu();
                    *action = TicketAction::UpdateAssign(Identifier::new(&ticket.adapter, ticket.id));
                };
            
            ui.separator();

            ui.menu_button("Change State", |ui| {
                for state in cache.states.keys() {
                    if ticket.state_name.eq(&state.name) && ticket.adapter.eq(&state.adapter) {
                        ui.button(RichText::new(state.name.clone()).strong().color(theme.foreground_marker2));
                    } else if ui.button(RichText::new(state.name.clone()).color(theme.foreground_secondary)).clicked() {
                        ui.close_menu();
                        *action = TicketAction::UpdateStateImmediate(Identifier::new(&ticket.adapter, ticket.id), state.name.clone());
                    }
                    
                };
            }).response.on_hover_text_at_pointer("Change the state of the right-clicked Ticket.");

            ui.menu_button("Add tags", |ui| {

                for tag in &ticket.tags {
                    if ui.button(RichText::new("🗙 ".to_owned() + tag.as_str()).color(theme.foreground_marker2)).clicked() {
                        ui.close_menu();
                        *action = TicketAction::RemoveTagImmediate(Identifier::new(&ticket.adapter, ticket.id), tag.clone());
                    };
                }

                ui.separator();

                // In same Adapter
                for tag in cache.tags.keys() {
                    if !ticket.tags.contains(&tag.name) 
                        && ticket.adapter == tag.adapter 
                        && ui.button(RichText::new(tag.name.clone()).color(theme.foreground_secondary))
                        .clicked() {
                        ui.close_menu();
                        *action = TicketAction::AddTagImmediate(Identifier::new(&ticket.adapter, ticket.id), tag.name.clone());
                    }
                };

                ui.separator();

                

                ui.menu_button(RichText::new("From other Adapters..").color(theme.foreground_secondary), |ui| {
                    for tag in cache.tags.keys() {
                        if !ticket.tags.contains(&tag.name) 
                        && ticket.adapter != tag.adapter 
                        && ui.button(RichText::new(tag.name.clone()).color(theme.foreground_secondary))
                        .clicked() {
                            ui.close_menu();
                            *action = TicketAction::AddTagImmediate(Identifier::new(&ticket.adapter, ticket.id), tag.name.clone());
                        }
                    };
                });

                ui.add_space(font_size / 2.0);

                ui.text_edit_singleline(&mut "custom_tag");
            }).response.on_hover_text_at_pointer("Add or remove Tags on the right-clicked Ticket. Add a custom Tag in the input at the bottom.");

            ui.separator();

            if ui.button("Move to Bucket...")
                .on_hover_text_at_pointer("Moves the right-clicked Ticket to a different Bucket within the Ticket Adapter.")
                .clicked() {
                    ui.close_menu();
                    *action = TicketAction::UpdateBucket(Identifier::new(&ticket.adapter, ticket.id));
                };

            if ui.button("Move to Adapter...")
                .on_hover_text_at_pointer("Moves the right-clicked Ticket to an entirely different Ticket Adapter.")
                .clicked() {
                    ui.close_menu();
                    *action = TicketAction::UpdateAdapter(Identifier::new(&ticket.adapter, ticket.id));
                };

            if ui.button("Clone...")
                .on_hover_text_at_pointer("Clone the right-clicked Ticket, and show edit Window.")
                .clicked() {
                    ui.close_menu();
                    *action = TicketAction::Clone(Identifier::new(&ticket.adapter, ticket.id));
                };

            if ui.button("New with Bucket...")
                .on_hover_text_at_pointer("Creates a new Ticket with the same Bucket and Adapter as the right-clicked Ticket.")
                .clicked() {
                    ui.close_menu();
                    *action = TicketAction::NewInBucket(Identifier::new(&ticket.adapter, ticket.bucket_id));
                };

            if ui.button(RichText::new("Delete Ticket").color(theme.foreground_marker2))
                .on_hover_text_at_pointer("Delete the right-clicked Ticket.")
                .clicked() {
                    ui.close_menu();
                    *action = TicketAction::Delete(Identifier::new(&ticket.adapter, ticket.id));
                };
        });
    }

    pub fn update_ticket_list(
        ui: &mut Ui, 
        ticket: &Ticket, 
        theme: &UITheme, 
        icon: Option<TextureHandle>,
        cache: &UICache
    ) -> TicketAction {
        let mut action = TicketAction::None;

        let half_font = theme.font_size as f32 / 2.0;
        let font_size = theme.font_size as f32;

        let mut main_group = Frame::group(ui.style());

        let mut ticket_background = theme.background_secondary;
        let mut title_color = theme.foreground_primary;

        if ticket.assigned_to.contains(&cache.username) {
            ticket_background = theme.background_tertiary;
            title_color = theme.foreground_marker2;
        };

        main_group = main_group.fill(ticket_background);

        main_group = main_group.inner_margin(Margin::same(half_font));
        let response = main_group.show(ui, |ui| {
            ui.vertical(|ui| {
                let width = ui.available_width() - half_font;
                ui.set_width_range(width ..= width);

                ui.horizontal_wrapped(|ui| {

                    match icon {
                        Some(mut found_icon) => {

                            let mut image = Image::new(&mut found_icon, Vec2 { x: font_size, y: font_size });
                            image = image.tint(title_color);
                            ui.add(image).on_hover_text_at_pointer(&ticket.adapter);
                        },
                        None => {ui.add_space(font_size);},
                    };

                    let state_button = Button::new(RichText::new(&ticket.state_name).strong().size(font_size).color(title_color));

                    if match cache.states.get(&StateIdentifier::new(&ticket.adapter, &ticket.state_name)) {
                        Some(description) => ui.add(state_button).on_hover_text_at_pointer(description),
                        None => ui.add(state_button),
                    }.clicked() {
                        action = TicketAction::UpdateState(Identifier::new(&ticket.adapter, ticket.id));
                    };

                    ui.add_space(half_font);

                    if ui.add(Button::new(RichText::new(&ticket.title)
                            .heading()
                            .color(title_color)
                            .size(font_size))
                        .frame(false))
                        .on_hover_text_at_pointer("Double-click title to edit Ticket.")
                        .double_clicked() {
                            action = TicketAction::Edit(Identifier::new(ticket.adapter.clone(), ticket.id));
                        };

                    for tag in &ticket.tags {
                        match cache.tags.get(&TagCacheKey::new(tag.clone(), ticket.adapter.clone())) {
                            Some(tag_colors) => {
                                let mut tag_button = Button::new(RichText::new("⏺").color(tag_colors[1]));
                                tag_button = tag_button.fill(tag_colors[0]);
                                ui.add(tag_button).on_hover_text_at_pointer(tag).context_menu(|ui| {
                                    if ui.button(RichText::new("Remove from Ticket").color(theme.foreground_marker2)).clicked() {
                                        action = TicketAction::RemoveTagImmediate(Identifier::new(ticket.adapter.clone(), ticket.id), tag.clone());
                                        ui.close_menu();
                                    };

                                    if ui.button(RichText::new("Remove from Adapter").color(theme.foreground_marker2)).clicked() {

                                        let mut tag_ref = Tag::default().with_name(tag.clone());
                                        tag_ref.adapter = ticket.adapter.clone();

                                        action = TicketAction::DropTagImmediate(tag_ref);
                                        ui.close_menu();
                                    }
                                });
                            },
                            None => {
                                let response = ui.button("⏺").on_hover_text_at_pointer(tag.to_string() + "; Click to add to Adapter");
                                if response.clicked() {
                                    action = TicketAction::NewTag(Identifier::new(ticket.adapter.clone(), tag.clone()));
                                };

                                response.context_menu(|ui| {
                                    if ui.button(RichText::new("Remove from Ticket").color(theme.foreground_marker2)).clicked() {
                                        action = TicketAction::RemoveTagImmediate(Identifier::new(ticket.adapter.clone(), ticket.id), tag.clone());
                                        ui.close_menu();
                                    };

                                    if ui.button(RichText::new("Add to Adapter")).clicked() {
                                        action = TicketAction::NewTag(Identifier::new(ticket.adapter.clone(), tag.clone()));
                                        ui.close_menu();
                                    };
                                });
                            }
                        };
                    }
                });
            });
        }).response;

        UserInterface::update_ticket_context_menu(response, &mut action, ui, ticket, theme, cache);

        action
    }

    pub fn update_ticket_half(
        ui: &mut Ui, 
        ticket: &Ticket, 
        theme: &UITheme, 
        icon: Option<TextureHandle>, 
        cache: &mut UICache
    ) -> TicketAction {

        let font_size = theme.font_size as f32;
        let half_font = theme.font_size as f32 / 2.0;
        let heading_size = theme.font_size as f32 * 1.5;
        //let width = ui.available_width() - (heading_size);

        let mut action = TicketAction::None;

        let mut main_group = Frame::group(ui.style());

        let mut ticket_background = theme.background_secondary;
        let mut title_color = theme.foreground_primary;

        if ticket.assigned_to.contains(&cache.username) {
            ticket_background = theme.background_tertiary;
            title_color = theme.foreground_marker2;
        };

        main_group = main_group.fill(ticket_background);

        main_group = main_group.inner_margin(Margin::same(half_font));
        let response = main_group.show(ui, |ui| {

            ui.vertical(|ui| {
                let width = ui.available_width() - half_font;
                ui.set_width_range(width ..= width);

                ui.horizontal_wrapped(|ui| {

                    match icon {
                        Some(mut found_icon) => {

                            let mut image = Image::new(&mut found_icon, Vec2 { x: font_size, y: font_size });
                            image = image.tint(title_color);
                            ui.add(image).on_hover_text_at_pointer(&ticket.adapter);
                        },
                        None => {ui.add_space(font_size);},
                    };

                    let state_button = Button::new(RichText::new(&ticket.state_name).strong().size(font_size).color(title_color));

                    if match cache.states.get(&StateIdentifier::new(&ticket.adapter, &ticket.state_name)) {
                        Some(description) => ui.add(state_button).on_hover_text_at_pointer(description),
                        None => ui.add(state_button),
                    }.clicked() {
                        action = TicketAction::UpdateState(Identifier::new(&ticket.adapter, ticket.id));
                    };

                    ui.add_space(half_font);

                    if ui.add(Button::new(RichText::new(&ticket.title)
                            .heading()
                            .color(title_color)
                            .size(font_size))
                        .frame(false))
                        .on_hover_text_at_pointer("Double-click title to edit Ticket.")
                        .double_clicked() {
                            action = TicketAction::Edit(Identifier::new(ticket.adapter.clone(), ticket.id));
                        };
                });
                
                ScrollArea::new([true, true])
                        .id_source(format!("scroll_{}::{}", ticket.id, ticket.adapter))
                        .auto_shrink([false, true])
                        .max_width(ui.available_width())
                        .max_height(heading_size * 5.0)
                        .min_scrolled_height(heading_size * 5.0)
                        .show(ui, |ui| {
                            CommonMarkViewer::new(format!("viewer_{}::{}", ticket.id, ticket.adapter)).show(ui, &mut cache.commonmark, &ticket.description);
                        });

                ui.horizontal_wrapped(|ui| {

                    for tag in &ticket.tags {
                        match cache.tags.get(&TagCacheKey::new(tag.clone(), ticket.adapter.clone())) {
                            Some(tag_colors) => {
                                let mut tag_button = Button::new(RichText::new(tag).color(tag_colors[1]));

                                tag_button = tag_button.fill(tag_colors[0]);
                                ui.add(tag_button).on_hover_text_at_pointer(tag).context_menu(|ui| {
                                    if ui.button(RichText::new("Remove from Ticket").color(theme.foreground_marker2)).clicked() {
                                        action = TicketAction::RemoveTagImmediate(Identifier::new(ticket.adapter.clone(), ticket.id), tag.clone());
                                        ui.close_menu();
                                    };

                                    if ui.button(RichText::new("Remove from Adapter").color(theme.foreground_marker2)).clicked() {

                                        let mut tag_ref = Tag::default().with_name(tag.clone());
                                        tag_ref.adapter = ticket.adapter.clone();

                                        action = TicketAction::DropTagImmediate(tag_ref);
                                        ui.close_menu();
                                    }
                                });
                            },
                            None => {
                                let response = ui.button(tag).on_hover_text_at_pointer("Click to add to Adapter");
                                if response.clicked() {
                                    action = TicketAction::NewTag(Identifier::new(ticket.adapter.clone(), tag.clone()));
                                };

                                response.context_menu(|ui| {
                                    if ui.button(RichText::new("Remove from Ticket").color(theme.foreground_marker2)).clicked() {
                                        action = TicketAction::RemoveTagImmediate(Identifier::new(ticket.adapter.clone(), ticket.id), tag.clone());
                                        ui.close_menu();
                                    };

                                    if ui.button(RichText::new("Add to Adapter")).clicked() {
                                        action = TicketAction::NewTag(Identifier::new(ticket.adapter.clone(), tag.clone()));
                                        ui.close_menu();
                                    };
                                });
                            }
                        };
                    }
                })
            });
        }).response;

        UserInterface::update_ticket_context_menu(response, &mut action, ui, ticket, theme, cache);

        action
    }


    pub fn update_ticket_regular(
        ui: &mut Ui, 
        ticket: &Ticket, 
        theme: &UITheme, 
        icon: Option<TextureHandle>, 
        cache: &mut UICache
    ) -> TicketAction {

        let half_font = theme.font_size as f32 / 2.0;
        let double_font = theme.font_size as f32 * 2.0;
        let heading_size = theme.font_size as f32 * 1.5;
        //let width = ui.available_width() - (heading_size);

        let mut action = TicketAction::None;

        let mut main_group = Frame::group(ui.style());
        let mut ticket_background = theme.background_secondary;
        let mut title_color = theme.foreground_primary;

        if ticket.assigned_to.contains(&cache.username) {
            ticket_background = theme.background_tertiary;
            title_color = theme.foreground_marker2;
        };
        

        main_group = main_group.fill(ticket_background);

        main_group = main_group.inner_margin(Margin::same(double_font));
        let response = main_group.show(ui, |ui| {

            ui.vertical(|ui| {
                let width = ui.available_width() - half_font;
                ui.set_width_range(width ..= width);

                ui.horizontal_wrapped(|ui| {

                    match icon {
                        Some(mut found_icon) => {

                            let mut image = Image::new(&mut found_icon, Vec2 { x: double_font, y: double_font });
                            image = image.tint(title_color);
                            ui.add(image).on_hover_text_at_pointer(&ticket.adapter);
                        },
                        None => {ui.add_space(double_font);},
                    };

                    let state_button = Button::new(RichText::new(&ticket.state_name).strong().size(heading_size).color(title_color));

                    if match cache.states.get(&StateIdentifier::new(&ticket.adapter, &ticket.state_name)) {
                        Some(description) => ui.add(state_button).on_hover_text_at_pointer(description),
                        None => ui.add(state_button),
                    }.clicked() {
                        action = TicketAction::UpdateState(Identifier::new(&ticket.adapter, ticket.id));
                    };

                    ui.add_space(half_font);

                    if ui.add(Button::new(RichText::new(&ticket.title)
                            .heading()
                            .color(title_color)
                            .size(heading_size))
                        .frame(false))
                        .on_hover_text_at_pointer("Double-click title to edit Ticket.")
                        .double_clicked() {
                            action = TicketAction::Edit(Identifier::new(ticket.adapter.clone(), ticket.id));
                        }; 
                });

                //ui.add_space(theme.font_size as f32);
                ui.add_space(half_font);
                //ui.monospace(&ticket.description);

                
                ScrollArea::new([true, true])
                        .id_source(format!("scroll_{}::{}", ticket.id, ticket.adapter))
                        .auto_shrink([false, true])
                        .max_width(ui.available_width())
                        .max_height(double_font * 10.0)
                        .min_scrolled_height(double_font * 10.0)
                        .show(ui, |ui| {
                            CommonMarkViewer::new(format!("viewer_{}::{}", ticket.id, ticket.adapter)).show(ui, &mut cache.commonmark, &ticket.description);
                        });

                
                //ui.add_space(half_font);

                ui.horizontal_wrapped(|ui| {

                    for tag in &ticket.tags {
                        match cache.tags.get(&TagCacheKey::new(tag.clone(), ticket.adapter.clone())) {
                            Some(tag_colors) => {
                                let mut tag_button = Button::new(RichText::new(tag).color(tag_colors[1]));

                                tag_button = tag_button.fill(tag_colors[0]);
                                ui.add(tag_button).on_hover_text_at_pointer(tag).context_menu(|ui| {
                                    if ui.button(RichText::new("Remove from Ticket").color(theme.foreground_marker2)).clicked() {
                                        action = TicketAction::RemoveTagImmediate(Identifier::new(ticket.adapter.clone(), ticket.id), tag.clone());
                                        ui.close_menu();
                                    };

                                    if ui.button(RichText::new("Remove from Adapter").color(theme.foreground_marker2)).clicked() {

                                        let mut tag_ref = Tag::default().with_name(tag.clone());
                                        tag_ref.adapter = ticket.adapter.clone();

                                        action = TicketAction::DropTagImmediate(tag_ref);
                                        ui.close_menu();
                                    }
                                });
                            },
                            None => {

                                let response = ui.button(tag).on_hover_text_at_pointer("Click to add as new Tag");
                                if response.clicked() {
                                    action = TicketAction::NewTag(Identifier::new(ticket.adapter.clone(), tag.clone()));
                                };

                                response.context_menu(|ui| {
                                    if ui.button(RichText::new("Remove from Ticket").color(theme.foreground_marker2)).clicked() {
                                        action = TicketAction::RemoveTagImmediate(Identifier::new(ticket.adapter.clone(), ticket.id), tag.clone());
                                        ui.close_menu();
                                    };

                                    if ui.button(RichText::new("Add to Adapter")).clicked() {
                                        action = TicketAction::NewTag(Identifier::new(ticket.adapter.clone(), tag.clone()));
                                        ui.close_menu();
                                    };
                                });
                            }
                        };
                    }
                })
            });

        }).response;

        UserInterface::update_ticket_context_menu(response, &mut action, ui, ticket, theme, cache);

        action
    }

}