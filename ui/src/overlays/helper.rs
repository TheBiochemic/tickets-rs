use std::{collections::{hash_map::RandomState, HashMap}, ops::Deref, sync::{Arc, Mutex}};

use chrono::{DateTime, Utc, Datelike, Duration, Timelike};
use eframe::Theme;
use egui::{Ui, Color32, Layout, Align, Vec2, Label, style::Margin, Frame, ComboBox, Separator, Button, ScrollArea, TextEdit, RichText, DragValue, TextStyle, menu, TextureHandle, ColorImage, Image, Area, color_picker::{color_picker_color32, Alpha}, color::Hsva, Slider, Visuals, TextBuffer, Checkbox};
use tickets_rs_core::{Bucket, Ticket, Tag, TicketProvider, Config, ToConfig, StateIdentifier};
use crate::{UITheme, overlays::{OverlayAction, DialogOptions}, Overlay, UserInterface, UIHelper, UIController, TagCacheKey};


pub struct OverlayHelper;

impl OverlayHelper {

    pub fn helper_update_warning(ui: &mut Ui, ui_theme: &UITheme, warn_text: &str) {
        let font_size = ui_theme.font_size as f32;

        Frame::group(ui.style())
            .inner_margin(Margin::same(font_size / 2.0))
            .fill(ui_theme.background_error).show(ui, |ui| {
                ui.set_min_width(ui.available_width() - 8.0);
                ui.set_max_width(ui.available_width() - 8.0);

                ui.heading(warn_text);
                
            });
    }

    pub fn helper_update_errors(ui: &mut Ui, ui_theme: &UITheme, errors: &Vec<(String, String)>) {
        let font_size = ui_theme.font_size as f32;

        if !errors.is_empty() {

            ui.separator();
            OverlayHelper::helper_update_small_spacer(ui, ui_theme);

            Frame::group(ui.style())
            .inner_margin(Margin::same(font_size / 2.0))
            .fill(ui_theme.background_error).show(ui, |ui| {
                ui.set_min_width(ui.available_width() - 8.0);
                ui.set_max_width(ui.available_width() - 8.0);

                ui.heading("Errors");
                ui.add_space((ui_theme.font_size as f32) / 2.0);

                for error in errors {
                    ui.group(|ui| {
                        ui.set_min_width(ui.available_width() - 8.0);
                        ui.set_max_width(ui.available_width() - 8.0);
                        ui.set_min_height(font_size * 1.0);
                        ui.set_max_height(font_size * 1.0);

                        ui.with_layout(Layout::left_to_right(Align::Min).with_main_wrap(true), |ui| {
                            ui.add_sized(Vec2{x: font_size * 5.0, y: font_size}, Label::new(format!("{}:", &error.0)));

                            ui.with_layout(Layout::left_to_right(Align::Min).with_main_wrap(true), |ui| {
                                ui.label(&error.1);
                            });
                        });
                    });
                    
                    ui.add_space(font_size / 4.0);
                }
                
            });
        }
    }

    /**
     * Returns true, if the random  color button has been pressed
     */
    pub fn helper_update_color(ui: &mut Ui, ui_theme: &UITheme, label_text: &str, mut color: &mut Color32) {
        let font_size = ui_theme.font_size as f32;
        let mut hsva = Hsva::from(*color);

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            // ui.set_max_height(font_size * 1.5);
            ui.set_max_width(ui.available_width() * 0.75);
            // ui.add_sized(Vec2{ x: ui.available_width() * 0.75, y: font_size * 1.5 }, TextEdit::singleline(title));
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                //ui.color_edit_button_srgba(&mut color);
                ui.vertical_centered_justified(|ui| {

                    ui.spacing_mut().slider_width = ui.available_width() * 1.00 - font_size * 10.0 ;

                    let mut changes = 0;

                    let button = Button::new(RichText::new("").size(font_size)).fill(*color);

                    let button_response = UIHelper::extend_tooltip(ui.add(button), ui_theme, "Click to generate new random color.");

                    if button_response.clicked() {

                        let (hue, sat, val) = Tag::generate_random_color(0.0 .. 1.0, 0.0 .. 1.0, 0.0 .. 1.0);
                        hsva.h = hue;
                        hsva.s = sat;
                        hsva.v = val;
                        changes += 1;

                    };

                    if ui.add(Slider::new(&mut hsva.h, 0.0..=1.0).text("Hue")).changed() {
                        changes += 1;
                    };

                    if ui.add(Slider::new(&mut hsva.s, 0.0..=1.0).text("Saturation")).changed() {
                        changes += 1;
                    };

                    if ui.add(Slider::new(&mut hsva.v, 0.0..=1.0).text("Value")).changed() {
                        changes += 1;
                    };

                    if changes > 0 {
                        *color = Color32::from(hsva);
                    };
                    
                });

                color

            });
            
            ui.add_space(font_size);
            ui.label(label_text);
        });
    }

    pub fn helper_update_adapter(ui: &mut Ui, ui_theme: &UITheme, adapter: &mut String, adapters: &Vec<(String, String)>) {
        let font_size = ui_theme.font_size as f32;

        let mut adapter_text = format!("Unknown Adapter ({})", *adapter);

        for adapter_pair in adapters {
            if *adapter == adapter_pair.0 {
                adapter_text = format!("{} ({})", adapter_pair.1, adapter_pair.0);
            }
        }

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.set_max_height(font_size * 1.5);
            ComboBox::from_id_source("adapter_dropdown")
                .selected_text(adapter_text)
                .width(ui.available_width() * 0.75 - 8.0)
                .show_ui(ui, |ui| {
                    for adapter_pair in adapters {
                        ui.selectable_value(adapter, adapter_pair.0.to_string(), format!("{} ({})", adapter_pair.1, adapter_pair.0).as_str());
                    }
                }
            );
            ui.add_space(font_size);
            ui.label("Adapter:");
        });
    }

    pub fn helper_update_bucket(ui: &mut Ui, ui_theme: &UITheme, bucket_id: &mut i64, buckets: &Vec<Bucket>, adapter: &String) {
        let font_size = ui_theme.font_size as f32;

        let mut bucket_text = format!("Unknown Bucket ({})", bucket_id);

        for bucket in buckets {
            if *bucket_id == bucket.identifier.id && bucket.identifier.adapter.eq(adapter) {
                bucket_text = format!("{} ({})", bucket.name, bucket.identifier.id);
            }
        }

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.set_max_height(font_size * 1.5);
            ComboBox::from_id_source("bucket_dropdown")
                .selected_text(bucket_text)
                .width(ui.available_width() * 0.75 - 8.0)
                .show_ui(ui, |ui| {
                    for bucket in buckets {
                        if bucket.identifier.adapter.eq(adapter) {
                            ui.selectable_value(bucket_id, bucket.identifier.id, format!("{} ({})", bucket.name, bucket.identifier.id).as_str());
                        }
                        
                    }
                }
            );
            ui.add_space(font_size);
            ui.label("Bucket:");
        });
    }

    pub fn helper_update_small_spacer(ui: &mut Ui, ui_theme: &UITheme) {
        ui.add_space(ui_theme.font_size as f32 / 4.0);
    }

    pub fn helper_update_spacer(ui: &mut Ui, ui_theme: &UITheme) {
        ui.add_space(ui_theme.font_size as f32);
    }

    pub fn helper_update_card(ui: &mut Ui, ui_theme: &UITheme, label: String, body: impl FnOnce(&mut Ui) -> ()) {
        let font_size = ui_theme.font_size as f32;

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.group(|ui| {

                ui.set_max_width((ui.available_width() * 0.75) - 4.0);
                ui.with_layout(Layout::top_down(Align::Min), body);

            });

            ui.add_space(font_size);
            ui.label(label);
        });
    }

    pub fn helper_update_section_collapsing(ui: &mut Ui, ui_theme: &UITheme, label: &str, open: bool, body: impl FnOnce(&mut Ui) -> ()) {
        ui.add_space(ui_theme.font_size as f32);

        let id = ui.make_persistent_id(label);
        
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, open)
        .show_header(ui, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                ui.set_max_height(ui_theme.font_size as f32);
                ui.label(label);
                ui.with_layout(Layout::bottom_up(Align::Max), |ui| {
                    ui.add(Separator::default().horizontal());
                });
            });
        })
        .body_unindented(body);
    }

    pub fn helper_update_dialog_buttons(ui: &mut Ui, ui_theme: &UITheme, button_text: Option<String>) -> DialogOptions {
        let font_size = ui_theme.font_size as f32;
        let mut action = DialogOptions::Nothing;
        let available_space = ui.available_width();

        ui.add_space(ui_theme.font_size as f32);
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.set_max_height(font_size * 1.5);

            if let Some(button_text) = button_text {

                if ui.add_sized(Vec2{x: font_size * 10.0, y: font_size * 1.5}, Button::new(button_text)).clicked() {
                    action = DialogOptions::Confirm;
                }
                ui.add_space(available_space - font_size * 21.0);
    
                if ui.add_sized(Vec2{x: font_size * 10.0, y: font_size * 1.5}, Button::new("Abort")).clicked() {
                    action = DialogOptions::Close;
                }

            } else {

                if ui.add_sized(Vec2{x: font_size * 10.0, y: font_size * 1.5}, Button::new("Close")).clicked() {
                    action = DialogOptions::Close;
                }
            }
            
        });

        action
    }

    pub fn helper_update_header(ui: &mut Ui, ui_theme: &UITheme, heading: &str) {
        ui.heading(heading);
        ui.add_space(ui_theme.font_size as f32);
    }

    pub fn helper_update_assigned(ui: &mut Ui, ui_theme: &UITheme, assigned_to: &mut String, assigned_text: &mut String, username: &String) {

        let assignees_clone = assigned_to.clone();
        let mut assignees = assignees_clone.split(",").map(|elem| {elem.trim().to_string()}).collect::<Vec<String>>();
        let font_size = ui_theme.font_size as f32;

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.set_max_height(font_size * 10.0);
            ui.set_max_width(ui.available_width() * 0.75);
            ScrollArea::new([false, true])
                .id_source("assigned_scroll_area")
                .auto_shrink([false, true])
                .show(ui, |ui| {

                    ui.group(|ui| {

                        ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {

                            let cols = (1.0_f32).max(ui.available_width() / (font_size * 10.0)) as i32;
                            let mut current_col = 0;
    
                            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {

                                ui.set_max_height(font_size * 1.5);
                                if ui.add_sized(
                                    Vec2{ x: ui.available_width() * 0.75, y: font_size * 1.5 }, 
                                    TextEdit::singleline(assigned_text))
                                    .on_hover_text_at_pointer("Add multiple users at once by separating them with a comma.").lost_focus() && 
                                    ui.input().key_pressed(egui::Key::Enter) {
                                        assignees.push(assigned_text.trim().to_string());
                                        *assigned_to = assignees.join(", ");
                                        assigned_text.clear();
                                }
                                ui.add_space(font_size / 2.0);
                                ui.label("New:");
                            });

                            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                ui.menu_button("More Options", |ui| {

                                    if ui.button("assign me").clicked() {
                                        if !assignees.contains(&username.trim().to_string()) {
                                            assignees.push(username.trim().to_string());
                                            *assigned_to = assignees.join(", ");
                                        }
                                        ui.close_menu();
                                    }

                                    if ui.button("remove all").clicked() {
                                        assignees.clear();
                                        *assigned_to = String::default();
                                        ui.close_menu();
                                    }
                                })
                            });
    
                            ui.separator();
    
                            egui::Grid::new("assigned_to_grid")
                                .min_col_width(ui.available_width() / (cols as f32))
                                .max_col_width(ui.available_width() / (cols as f32))
                                .spacing(Vec2{x: 2.0, y: 2.0})
                                .show(ui, |ui| {

                                for (pos, assignee) in assignees.clone().iter().enumerate() {
    
                                    let assignee_cleared = assignee.trim();
                                    if assignee_cleared.is_empty() {
                                        continue;
                                    }

                                    if current_col >= cols {
                                        ui.end_row();
                                        current_col = 0;
                                    }
    
                                    let mut assigned_to_button = Button::new(
                                        RichText::new("🗙 ".to_owned() + assignee_cleared)
                                        .color(ui_theme.foreground_marker2)
                                        .size(font_size));

                                    if ui.add_sized(Vec2{x: ui.available_width() - 8.0, y: font_size * 1.20}, assigned_to_button).clicked() {
                                        assignees.remove(pos);
                                        *assigned_to = assignees.join(", ");
                                    };

                                    current_col += 1;
                                }
                            });
                        });
                    });
            });

            ui.add_space(font_size);
            ui.label("Assigned:");

        });

    }

    pub fn helper_update_due(ui: &mut Ui, ui_theme: &UITheme, due_date: &mut DateTime<Utc>) -> Option<u64> {

        let timezone = chrono::Local::now().offset().clone();
        let font_size = ui_theme.font_size as f32;
        let date_view = due_date.with_timezone(&timezone).naive_local();

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            //ui.set_max_height(font_size * 3.0);
            //ui.set_max_width(available_width);

            ui.group(|ui| {

                ui.set_max_width((ui.available_width() * 0.75) - 3.0);
                ui.with_layout(Layout::top_down(Align::Min), |ui| {

                    let mut year: i32 = date_view.year();
                    let mut month: i32 = date_view.month() as i32;
                    let mut day: i32 = date_view.day() as i32;
                    let mut hour: i32 = date_view.hour() as i32;
                    let mut minute: i32 = date_view.minute() as i32;

                    let mut year_changed = false;
                    let mut month_changed = false;
                    let mut day_changed = false;
                    let mut hour_changed = false;
                    let mut minute_changed = false;

                    if ui.available_width() > 35.0 * font_size {

                        let row_width = ui.available_width() * 0.75;
                        let col_width = row_width / 6.0;
                        let spacing = font_size / 4.0;

                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {

                            if ui.add_sized(
                                Vec2::new(col_width - spacing, font_size),
                                DragValue::new(&mut minute)
                                    .speed(0.25)
                                    .custom_formatter(|n, _| format!("{:0>2}", n as i32))
                            ).on_hover_text_at_pointer("The minutes of that Property. Drag up or down to change it.")
                            .changed() {
                                minute = minute / 5 * 5;
                                minute_changed = true;
                            };

                            ui.add_space(spacing);

                            if ui.add_sized(
                                Vec2::new(col_width, font_size),
                                DragValue::new(&mut hour)
                                    .speed(0.05)
                                    .custom_formatter(|n, _| format!("{:0>2}", n as i32))
                            ).on_hover_text_at_pointer("The hour (24h) of that Property. Drag up or down to change it.")
                            .changed() {
                                hour_changed = true;
                            };

                            ui.horizontal(|ui| {
                                ui.set_max_width(col_width - spacing * 4.0);
                                ui.add_sized(Vec2::new(col_width - spacing * 4.0, font_size), Label::new(":"));
                            });

                            if ui.add_sized(
                                Vec2::new(col_width - spacing, font_size),
                                DragValue::new(&mut day)
                                    .speed(0.1)
                                    .custom_formatter(|n, _| format!("{:0>2}", n as i32))
                            ).on_hover_text_at_pointer("The day of the month of that Property. Drag up or down to change it.")
                            .changed() {
                                day_changed = true;
                            };

                            ui.add_space(spacing);

                            if ui.add_sized(
                                Vec2::new(col_width - spacing, font_size),
                                DragValue::new(&mut month)
                                    .speed(0.05)
                                    .custom_formatter(|n, _| format!("{:0>2}", n as i32))
                            ).on_hover_text_at_pointer("The month of that Property. Drag up or down to change it.")
                            .changed() {
                                month_changed = true;
                            };

                            ui.add_space(spacing);

                            if ui.add_sized(
                                Vec2::new(col_width, font_size),
                                DragValue::new(&mut year)
                                    .speed(0.05)
                                    .custom_formatter(|n, _| format!("{:0>4}", n as i32))
                            ).on_hover_text_at_pointer("The year of that Property. Drag up or down to change it.")
                            .changed() {
                                year_changed = true;
                            };

                            ui.add_space(font_size / 2.0);
                            ui.label("Date/Time:")
    
                        });

                    } else {

                        let spacing = font_size / 4.0;
                        let row_width = ui.available_width() * 0.75;
                        let col_width = row_width / 3.3;

                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {

                            if ui.add_sized(
                                Vec2::new(col_width - spacing, font_size),
                                DragValue::new(&mut day)
                                    .speed(0.1)
                                    .custom_formatter(|n, _| format!("{:0>2}", n as i32))
                            ).on_hover_text_at_pointer("The day of the month of that Property. Drag up or down to change it.")
                            .changed() {
                                day_changed = true;
                            };

                            ui.add_space(spacing);

                            if ui.add_sized(
                                Vec2::new(col_width - spacing, font_size),
                                DragValue::new(&mut month)
                                    .speed(0.05)
                                    .custom_formatter(|n, _| format!("{:0>2}", n as i32))
                            ).on_hover_text_at_pointer("The month of that Property. Drag up or down to change it.")
                            .changed() {
                                month_changed = true;
                            };

                            ui.add_space(spacing);

                            if ui.add_sized(
                                Vec2::new(col_width, font_size),
                                DragValue::new(&mut year)
                                    .speed(0.05)
                                    .custom_formatter(|n, _| format!("{:0>4}", n as i32))
                            ).on_hover_text_at_pointer("The year of that Property. Drag up or down to change it.")
                            .changed() {
                                year_changed = true;
                            };

                            ui.add_space(font_size / 2.0);
                            ui.label("Date:")
                        });

                        let col_width = (row_width + font_size - spacing) / 2.2;

                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {

                            if ui.add_sized(
                                Vec2::new(col_width - spacing, font_size),
                                DragValue::new(&mut minute)
                                    .speed(0.25)
                                    .custom_formatter(|n, _| format!("{:0>2}", n as i32))
                            ).on_hover_text_at_pointer("The minutes of that Property. Drag up or down to change it.")
                            .changed() {
                                minute = minute / 5 * 5;
                                minute_changed = true;
                            };

                            ui.add_space(spacing);

                            if ui.add_sized(
                                Vec2::new(col_width, font_size),
                                DragValue::new(&mut hour)
                                    .speed(0.05)
                                    .custom_formatter(|n, _| format!("{:0>2}", n as i32))
                            ).on_hover_text_at_pointer("The hour (24h) of that Property. Drag up or down to change it.")
                            .changed() {
                                hour_changed = true;
                            };

                            ui.add_space(font_size / 2.0);
                            ui.label("Time:")
                        });

                        ui.separator();
                    }

                    if minute_changed {
                        if let Some(new_date_view) = date_view.with_minute(minute as u32) {
                            *due_date = new_date_view.and_local_timezone(timezone).unwrap().naive_utc().and_local_timezone(Utc).unwrap();
                        }
                    }

                    if hour_changed {
                        if let Some(new_date_view) = date_view.with_hour(hour as u32) {
                            *due_date = new_date_view.and_local_timezone(timezone).unwrap().naive_utc().and_local_timezone(Utc).unwrap();
                        }
                    }

                    if day_changed {
                        if let Some(new_date_view) = date_view.with_day(day as u32) {
                            *due_date = new_date_view.and_local_timezone(timezone).unwrap().naive_utc().and_local_timezone(Utc).unwrap();
                        }
                    }

                    if month_changed {
                        if let Some(new_date_view) = date_view.with_month(month as u32) {
                            *due_date = new_date_view.and_local_timezone(timezone).unwrap().naive_utc().and_local_timezone(Utc).unwrap();
                        }
                    }

                    if year_changed {
                        if let Some(new_date_view) = date_view.with_year(year) {
                            *due_date = new_date_view.and_local_timezone(timezone).unwrap().naive_utc().and_local_timezone(Utc).unwrap();
                        }
                    }

                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        ui.menu_button("More Options", |ui| {
    
                            if ui.button("set to today").clicked() {
                                *due_date = Utc::now();
                                ui.close_menu();
                            }
    
                            if ui.button("set to tomorrow").clicked() {
                                *due_date = Utc::now() + Duration::days(1);
                                ui.close_menu();
                            }

                            if ui.button("set to next week").clicked() {
                                *due_date = Utc::now() + Duration::days(7);
                                ui.close_menu();
                            }

                            ui.separator();

                            if ui.button("add day").clicked() {
                                *due_date += Duration::days(1);
                                ui.close_menu();
                            }

                            if ui.button("add week").clicked() {
                                *due_date += Duration::days(7);
                                ui.close_menu();
                            }

                        })
                    });

                });
            });

            ui.add_space(font_size);
            ui.label("Due at:");
        });

        None
    }

    pub fn helper_update_text(ui: &mut Ui, ui_theme: &UITheme, title: &mut String, button_label: &str) -> bool {

        let font_size = ui_theme.font_size as f32;
        let mut changed = false;

        
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.set_max_height(font_size * 1.5);
            if ui.add_sized(Vec2{ x: ui.available_width() * 0.75, y: font_size * 1.5 }, TextEdit::singleline(title)).changed() {
                changed = true;
            };
            ui.add_space(font_size);
            ui.label(button_label);
        });

        changed
    }

    pub fn helper_update_number(ui: &mut Ui, ui_theme: &UITheme, number: &mut i32, button_label: &str) -> bool {
        let font_size = ui_theme.font_size as f32;
        let mut changed = false;

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.set_max_height(font_size * 1.5);
            let drag_value = DragValue::new(number);
            changed = ui.add_sized(Vec2{ x: ui.available_width() * 0.75, y: font_size * 1.5 }, drag_value).changed();
            ui.add_space(font_size);
            ui.label(button_label);
        });

        changed
    }

    pub fn helper_update_number64(ui: &mut Ui, ui_theme: &UITheme, number: &mut i64, button_label: &str) -> bool {
        let font_size = ui_theme.font_size as f32;
        let mut changed = false;

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.set_max_height(font_size * 1.5);
            let drag_value = DragValue::new(number);
            changed = ui.add_sized(Vec2{ x: ui.available_width() * 0.75, y: font_size * 1.5 }, drag_value).changed();
            ui.add_space(font_size);
            ui.label(button_label);
        });

        changed
    }

    pub fn helper_update_theme(ui: &mut Ui, ui_theme: &UITheme, theme_ref: &mut UITheme, ui_controller: &mut UIController) {
        let font_size = ui_theme.font_size as f32;

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.set_max_height(font_size * 1.5);

            let mut changed_value = false;

            ComboBox::from_id_source("theme_dropdown")
                .selected_text(theme_ref.name())
                .width(ui.available_width() * 0.75 - 8.0)
                .show_ui(ui, |ui| {
                    for name in UITheme::names() {

                        let selected_value = UITheme::from_name(&name);
                        let mut response = ui.selectable_label(*theme_ref == selected_value, &name);
                        if response.clicked() {
                            theme_ref.merge_colors(&selected_value);
                            response.mark_changed();
                            changed_value = true;
                        }

                    }
                }
            );
            
            if changed_value {
                let mut style = ui.ctx().style().deref().clone();

                style.visuals = if theme_ref.base_theme == Theme::Dark {
                    Visuals::dark()
                } else {
                    Visuals::light()
                };
                ui.ctx().set_style(Arc::new(style));
                ui_controller.font_changed = true;
            };

            ui.add_space(font_size);
            ui.label("Theme:");
        });
    }

    pub fn helper_update_desc(ui: &mut Ui, ui_theme: &UITheme, description: &mut String, markdown: bool) {

        let font_size = ui_theme.font_size as f32;

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.set_max_height(font_size * 10.0);
            ui.set_max_width(ui.available_width() * 0.75);

            ui.group(|ui| {
                ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {


                    if markdown {
                        menu::bar(ui, |ui| {
                            ui.menu_button("Header", |ui| {
                                if ui.button("Primary").clicked() {
                                    *description += "# Primary Title\n";
                                    ui.close_menu();
                                };
        
                                if ui.button("Secondary").clicked() {
                                    *description += "## Secondary Title\n";
                                    ui.close_menu();
                                };
        
                                if ui.button("Tertiary").clicked() {
                                    *description += "### Tertiary Title\n";
                                    ui.close_menu();
                                };
                            });
            
                            ui.menu_button("Styling", |ui| {
                                if ui.button("Bold").clicked() {
                                    *description += "**Bold text**";
                                    ui.close_menu();
                                };
        
                                if ui.button("Italic").clicked() {
                                    *description += "*Italic text*";
                                    ui.close_menu();
                                };
        
                                if ui.button("Strikethrough").clicked() {
                                    *description += "~~Striked through text~~";
                                    ui.close_menu();
                                };
                            });
            
                            ui.menu_button("List", |ui| {
                                if ui.button("Ordered").clicked() {
                                    *description += "1. First Item\n2. Second Item\n3. Third Item\n";
                                    ui.close_menu();
                                };
        
                                if ui.button("Unordered").clicked() {
                                    *description += "- Item 1\n- Item 2\n- Item 3\n";
                                    ui.close_menu();
                                };
        
                                if ui.button("Checklist").clicked() {
                                    *description += "- [x] Item 1\n- [ ] Item 2\n- [ ] Item 3\n";
                                    ui.close_menu();
                                };
                            });
            
                            ui.menu_button("Code", |ui| {
                                if ui.button("Single").clicked() {
                                    *description += "`Single Code`";
                                    ui.close_menu();
                                };
        
                                if ui.button("Block").clicked() {
                                    *description += "```\nCode\nBlock\n```\n";
                                    ui.close_menu();
                                };
                            });
            
                            ui.menu_button("Table", |ui| {
                                if ui.button("2 Columns").clicked() {
                                    *description += "|Header 1|Header 2|\n";
                                    *description += "|--------|--------|\n";
                                    *description += "|Entry 1 |Entry 2 |\n";
                                    *description += "|Entry 3 |Entry 4 |\n";
                                    ui.close_menu();
                                };
        
                                if ui.button("3 Columns").clicked() {
                                    *description += "|Header 1|Header 2|Header 3|\n";
                                    *description += "|--------|--------|--------|\n";
                                    *description += "|Entry 1 |Entry 2 |Entry 3 |\n";
                                    *description += "|Entry 4 |Entry 5 |Entry 6 |\n";
                                    ui.close_menu();
                                };
        
                                if ui.button("4 Columns").clicked() {
                                    *description += "|Header 1|Header 2|Header 3|Header 4|\n";
                                    *description += "|--------|--------|--------|--------|\n";
                                    *description += "|Entry 1 |Entry 2 |Entry 3 |Entry 4 |\n";
                                    *description += "|Entry 5 |Entry 6 |Entry 7 |Entry 8 |\n";
                                    ui.close_menu();
                                };
        
                                if ui.button("5 Columns").clicked() {
                                    *description += "|Header 1|Header 2|Header 3|Header 4|Header 5|\n";
                                    *description += "|--------|--------|--------|--------|--------|\n";
                                    *description += "|Entry 1 |Entry 2 |Entry 3 |Entry 4 |Entry 5 |\n";
                                    *description += "|Entry 6 |Entry 7 |Entry 8 |Entry 9 |Entry 10|\n";
                                    ui.close_menu();
                                };
                            });
            
                            ui.menu_button("Other", |ui| {
                                if ui.button("Horizontal Line").clicked() {
                                    *description += "---\n";
                                    ui.close_menu();
                                };
        
                                if ui.button("Blockquote").clicked() {
                                    *description += "> This is a Blockquote\n";
                                    ui.close_menu();
                                };
        
                                if ui.button("Footnote").clicked() {
                                    *description += "[^1]";
                                    ui.close_menu();
                                };
        
                                if ui.button("Link").clicked() {
                                    *description += "[Linktitle](https://www.example.com)";
                                    ui.close_menu();
                                };
                            });
                        });
                    }
    
                    ScrollArea::new([false, true])
                        .auto_shrink([false, true])
                        .id_source("scroll_description")
                        .show(ui, |ui| {
    
                            ui.add_sized(
                                Vec2{x: ui.available_width(), y: ui.available_height()}, 
                                TextEdit::multiline(description).font(TextStyle::Monospace));
                    });
                });
            });
            

            ui.add_space(font_size);
            ui.label("Description:");
        });
    }

    pub fn helper_update_icon(
        ui: &mut Ui, 
        icon_textures: &mut HashMap<String, Option<TextureHandle>, RandomState>,
        icons: &mut HashMap<String, Option<ColorImage>, RandomState>,
        icon_name: &String,
        icon_size: f32) {
            let mut icon = UserInterface::load_texture(icon_textures, icons, ui, icon_name);
            if let Some(mut icon) = icon {
                let mut image = Image::new(&mut icon, Vec2::splat(icon_size));
                ui.add(image);
            } else {
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.set_min_size(Vec2::splat(icon_size));
                    ui.set_max_size(Vec2::splat(icon_size));
                });
            };
            
    }

    pub fn helper_update_state(ui: &mut Ui, ui_theme: &UITheme, state_name: &mut String, states: &HashMap<StateIdentifier, String>, adapter: &String) {
        let font_size = ui_theme.font_size as f32;

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.set_max_height(font_size * 1.5);
            ComboBox::from_id_source("state_dropdown")
                .selected_text(state_name.clone())
                .width(ui.available_width() * 0.75 - 8.0)
                .show_ui(ui, |ui| {
                    for state in states.keys() {
                        if adapter.eq(&state.adapter) {
                            ui.selectable_value(state_name, state.name.to_string(), state.name.clone());
                        }
                        
                    }
                }
            );
            ui.add_space(font_size);
            ui.label("State:");
        });
    }
    
    pub fn helper_update_tag(ui: &mut Ui, ui_theme: &UITheme, tag_text: &String, tag_color: &Color32, tag_color_back: &Color32) {
        let font_size = ui_theme.font_size as f32;

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.set_max_height(font_size * 1.5);
            ui.set_min_width(ui.available_width() * 0.75);
            ui.set_max_width(ui.available_width() * 0.75);
            ui.with_layout(Layout::top_down(Align::Center), |ui| {

                let button = Button::new(
                    RichText::new(tag_text)
                    .color(*tag_color)
                    .size(font_size))
                    .fill(*tag_color_back);
                ui.add(button);

            });
            ui.add_space(font_size);
            ui.label("Tag:");
        });
    }

    pub fn helper_update_extensions(ui: &mut Ui, ui_theme: &UITheme, ticket_provider: Arc<Mutex<TicketProvider>>, adapter_config: &mut Option<Config>) {
        let font_size = ui_theme.font_size as f32;

        match ticket_provider.lock() {
            Ok(lock) => {

            // Editing a currently open Adapter Config
            let mut abort_config = false;
            let mut finish_config = false;
            let mut change_config: Option<(String, String, String)> = None; //name, value, display_options
            match adapter_config {
                Some(config) => {

                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        ui.set_max_height(font_size * 10.0);
                        ui.set_max_width(ui.available_width() * 0.75);
            
                        ScrollArea::new([false, true])
                            .id_source("config_extensions_scroll_area")
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
            
                                ui.group(|ui| {
            
                                    ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                
                                        for option in config.iter() {

                                            match option.1.display_options().as_str() {
                                                "readonly_string" => {

                                                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {

                                                        let total_line_width = ui.available_width();

                                                        ui.group(|ui| {

                                                            ui.set_max_width(total_line_width * 0.75 - 10.0);
                                                            ui.set_min_width(total_line_width * 0.75 - 10.0);

                                                            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                                                                ui.add(Label::new(RichText::new(option.1.raw()).strong()).wrap(true));
                                                            });
                                                        });
                                                            

                                                        ui.add_space(font_size / 2.0);
                                                        ui.add(Label::new(option.0.clone() + ":").wrap(true));
                                                    });

                                                },
                                                "string" => {
                                                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {

                                                        let total_line_width = ui.available_width();

                                                        //ui.set_max_width(ui.available_width() * 0.8);
                                                        let mut text = option.1.raw().clone();

                                                        if ui.add_sized(
                                                            [total_line_width * 0.75, font_size], 
                                                            TextEdit::singleline(&mut text)
                                                        ).changed() {
                                                            change_config = Some((option.0.clone(), text, "string".to_string()));
                                                        }

                                                        ui.add_space(font_size / 2.0);
                                                        ui.add(Label::new(option.0.clone() + ":").wrap(true));
                                                    });
                                                },
                                                "bool" => {
                                                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {

                                                        let total_line_width = ui.available_width();

                                                        //ui.set_max_width(ui.available_width() * 0.8);
                                                        let mut toggle_value = match option.1.get::<bool>().clone() {
                                                            Some(val) => val,
                                                            None => false,
                                                        };

                                                        if ui.add_sized(
                                                            [total_line_width * 0.75, font_size], 
                                                            Checkbox::new(&mut toggle_value, "")
                                                        ).changed() {
                                                            change_config = Some((option.0.clone(), toggle_value.to_config(), "bool".to_string()));
                                                        }

                                                        ui.add_space(font_size / 2.0);
                                                        ui.add(Label::new(option.0.clone() + ":").wrap(true));
                                                    });
                                                },
                                                unmatched => {

                                                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {

                                                        let total_line_width = ui.available_width();

                                                        ui.group(|ui| {

                                                            ui.set_max_width(total_line_width * 0.75 - 10.0);
                                                            ui.set_min_width(total_line_width * 0.75 - 10.0);

                                                            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                                                                ui.add(Label::new(RichText::new(
                                                                    "??".to_string() + 
                                                                    option.0.as_str() + 
                                                                    "(" + unmatched.as_str() + 
                                                                    ") -> " +
                                                                    option.1.raw().as_str()
                                                                ).strong()).wrap(true));
                                                            });
                                                        });
                                                    });
                                                }
                                            }

                                        };

                                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                            if ui.button("Add Extension").clicked() {
                                                finish_config = true;
                                            }

                                            if ui.button("Abort").clicked() {
                                                abort_config = true;
                                            }
                                        });
                                    });
            
                                });
            
                        });
            
                        ui.add_space(font_size);
                        ui.label("Configure:");
                    });


                    match change_config {
                        Some(change) => {
                            config.put(change.0.as_str(), change.1.as_str(), change.2.as_str());
                        },
                        None => (),
                    };

                    if finish_config {
                        match lock.adapter_from_config(config, true) {
                            Ok(_) => abort_config = true,
                            Err(err) => println!("Wasn't able to create an adapter from Config due to {err}"),
                        }
                    }

                },
                None => (),
            };

            if abort_config {
                *adapter_config = None;
                abort_config = false;
            }


                // View and remove installed Adapters
                if lock.has_adapters() {

                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        ui.set_max_height(font_size * 10.0);
                        ui.set_max_width(ui.available_width() * 0.75);
            
                        ScrollArea::new([false, true])
                            .id_source("installed_extensions_scroll_area")
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
            
                                ui.group(|ui| {
            
                                    ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
            
                                            for adapter in lock.list_adapter_refs() {
                                                if ui.button("🗙 ".to_owned() + adapter.get_fancy_name().as_str()).clicked() {
                                                    lock.drop_adapter(adapter.get_name(), true);
                                                };
                                            }
                                    });
            
                                });
            
                        });
            
                        ui.add_space(font_size);
                        ui.label("Installed:");
                    });
                }

                // View and Add available Adapters
                let available_adapters = lock.list_available_adapter_types();
                if !available_adapters.is_empty() {
                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        ui.set_max_height(font_size * 10.0);
                        ui.set_max_width(ui.available_width() * 0.75);
            
                        ScrollArea::new([false, true])
                            .id_source("available_extensions_scroll_area")
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
            
                                ui.group(|ui| {
            
                                    ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
            
                                            for available_adapter in available_adapters {
                                                if ui.button(&available_adapter.fancy_name).clicked() {

                                                    if let Some(found_config) = lock.get_type_config(&available_adapter.name) {
                                                        *adapter_config = Some(found_config);
                                                    }

                                                };
                                            }
                                    });
            
                                });
            
                        });
            
                        ui.add_space(font_size);
                        ui.label("Available:");
                    });
                }
                
            },
            Err(err) => println!("Wasn't able to Lock Ticket Provider, due to {err}"),
        }
    }

    pub fn helper_update_tags(ui: &mut Ui, ui_theme: &UITheme, ticket_tags: &mut Vec<String>, tag_text: &mut String, adapter: &String, tags: &HashMap<TagCacheKey, [Color32; 2]>) {
        let font_size = ui_theme.font_size as f32;

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.set_max_height(font_size * 10.0);
            ui.set_max_width(ui.available_width() * 0.75);
            ScrollArea::new([false, true])
                .id_source("tags_scroll_area")
                .auto_shrink([false, true])
                .show(ui, |ui| {

                    ui.group(|ui| {

                        ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {

                            let cols = (1.0_f32).max(ui.available_width() / (font_size * 10.0)) as i32;
                            let mut current_col = 0;
    
                            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {

                                ui.set_max_height(font_size * 1.5);
                                if ui.add_sized(
                                    Vec2{ x: ui.available_width() * 0.75, y: font_size * 1.5 }, 
                                    TextEdit::singleline(tag_text)).lost_focus() && 
                                    ui.input().key_pressed(egui::Key::Enter) {
                                        ticket_tags.push(tag_text.clone());
                                        tag_text.clear();
                                }
                                ui.add_space(font_size / 2.0);
                                ui.label("New:");
                            });

                            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                ui.menu_button("More Options", |ui| {

                                    if ui.button("remove all").clicked() {
                                        ticket_tags.clear();
                                        ui.close_menu();
                                    }
                                })
                            });
    
                            ui.separator();
    
                            let mut add_separator = false;
                            egui::Grid::new("added_tags_grid")
                                .min_col_width(ui.available_width() / (cols as f32))
                                .max_col_width(ui.available_width() / (cols as f32))
                                .spacing(Vec2{x: 2.0, y: 2.0})
                                .show(ui, |ui| {
    
                                for tag in &ticket_tags.clone() {
    
                                    if current_col >= cols {
                                        ui.end_row();
                                        current_col = 0;
                                    }
    
                                    let mut tag_button = Button::new(
                                        RichText::new("🗙 ".to_owned() + tag.as_str())
                                        .color(ui_theme.foreground_marker2)
                                        .size(font_size));
                                    if ui.add_sized(Vec2{x: ui.available_width() - 8.0, y: font_size * 1.20}, tag_button).clicked() {
                                        ticket_tags.retain(|x| *x != *tag);
                                    };
                                    add_separator = true;
                                    current_col += 1;
                                }
                            });
            
                            if add_separator {
                                ui.separator();
                                current_col = 0;
                            }
    
                            egui::Grid::new("new_tags_grid")
                                .min_col_width(ui.available_width() / (cols as f32))
                                .max_col_width(ui.available_width() / (cols as f32))
                                .spacing(Vec2{x: 2.0, y: 2.0})
                                .show(ui, |ui| {
    
                                for tag in tags {
                                    if !ticket_tags.contains(&tag.0.name) && tag.0.adapter.eq(adapter) {
        
                                        if current_col >= cols {
                                            ui.end_row();
                                            current_col = 0;
                                        }
        
                                        let mut tag_button = Button::new(
                                            RichText::new(tag.0.name.clone())
                                            .color(tag.1[1])
                                            .size(font_size));
                                        
                                        tag_button = tag_button.fill(tag.1[0]);
        
                                        if ui.add_sized(Vec2{x: ui.available_width() - 8.0, y: font_size * 1.20}, tag_button).clicked() {
                                            ticket_tags.push(tag.0.name.clone());
                                        };
    
                                        current_col += 1;
                                    }
                                };
                            });

                            ui.add_space(font_size / 2.0);

                            current_col = 0;
                            egui::Grid::new("foreign_tags_grid")
                                .min_col_width(ui.available_width() / (cols as f32))
                                .max_col_width(ui.available_width() / (cols as f32))
                                .spacing(Vec2{x: 2.0, y: 2.0})
                                .show(ui, |ui| {
    
                                for tag in tags {
                                    if !ticket_tags.contains(&tag.0.name) && !tag.0.adapter.eq(adapter) {
        
                                        if current_col >= cols {
                                            ui.end_row();
                                            current_col = 0;
                                        }
        
                                        let tag_button = Button::new(
                                            RichText::new(tag.0.name.clone())
                                            .size(font_size));
                                        
                                        if ui.add_sized(Vec2{x: ui.available_width() - 8.0, y: font_size * 1.20}, tag_button)
                                           .on_hover_text_at_pointer("from Adapter \"".to_string() + &tag.0.adapter + "\"")
                                           .clicked() {
                                            ticket_tags.push(tag.0.name.clone());
                                        };
    
                                        current_col += 1;
                                    }
                                };
                            });
                        });

                    });
            });
            ui.add_space(font_size);
            ui.label("Tags:");
        });
    }

}
