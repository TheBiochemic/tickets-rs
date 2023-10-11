use std::{sync::{Arc, Mutex}, collections::{HashMap, BTreeMap}, rc::Rc, cell::RefCell, fs::File, path::Path, time::SystemTime};

use chrono::{DateTime, NaiveDateTime, Utc, TimeZone};
use eframe::IconData;
use std::fmt::Write;
use eframe::egui::{Ui, SelectableLabel, ColorImage, TextureHandle, Color32};
use tickets_rs_core::{AppConfig, TicketProvider, BucketPanelLocation, BucketPanelLocationType, Ticket, Tag, Bucket, AdapterError, TicketAdapter, State, FilterType, Filter, FilterIdentifier, StateIdentifier, BucketIdentifier};

use crate::{UserInterface, UITheme, Overlay, overlays::{NewTicketData, OverlayAction, NewTagData, WizardData, UpdateTicketData, NewStateData, NewBucketData, UpdateTicketDataBucket, UpdateTicketDataAssign, EditTicketData, PreferenceData, DeleteAdapterData, NewFilterData, DeleteFilterData, EditFilterData, DeleteBucketData, UpdateTicketDataAdapter}, UICache, user_interface::SidePanelAction, TagsCache, TagCacheKey};

use self::ticket_actions::TicketAction;

type ImageData = (Vec<u8>, u32, u32);

#[derive(Clone)]
pub struct BucketPanelEntry {
    pub label: String,
    pub adapter: String,
    pub filter: String,
    pub entry_type: FilterType,
}

pub mod ticket_actions {
    use std::marker::PhantomData;

    use tickets_rs_core::{Bucket, Ticket, Tag};

    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
    pub struct Identifier<ID, NAME> {
        pub adapter: String,
        pub id: ID,
        marker: PhantomData<NAME>
    }

    impl<ID, NAME> Identifier<ID, NAME> {
        pub fn new<S: Into<String>>(adapter: S, id: ID) -> Self {
            Identifier { adapter: adapter.into(), id , marker: PhantomData}
        }
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
    pub enum TicketAction {
        NewInBucket(Identifier<u64, Bucket>),
        Edit(Identifier<i64, Ticket>),
        Delete(Identifier<i64, Ticket>),
        UpdateDetails(Identifier<i64, Ticket>),
        UpdateAssign(Identifier<i64, Ticket>),
        UpdateState(Identifier<i64, Ticket>),
        UpdateStateImmediate(Identifier<i64, Ticket>, String), //State_name
        AddTagImmediate(Identifier<i64, Ticket>, String), //Tag_name
        RemoveTagImmediate(Identifier<i64, Ticket>, String), //Tag_name
        DropTagImmediate(Tag),
        NewTag(Identifier<String, Tag>), 
        Clone(Identifier<i64, Ticket>),
        UpdateBucket(Identifier<i64, Ticket>),
        UpdateAdapter(Identifier<i64, Ticket>),
        None
    }
}

#[derive(PartialEq)]
pub enum TicketViewMode {
    Regular,
    Half,
    List
}


// In a Bucket Panel Folder basically all entries are stored, that start a filter
#[derive(Clone)]
pub struct BucketPanelFolder {
    pub label: String,
    pub adapter: String,
    pub entries: Vec<BucketPanelEntry>,
    pub is_open: bool
}

pub struct UIController {
    pub running: bool,
    pub show_sidebar: bool,
    pub font_changed: bool,
    pub ticket_view_mode: TicketViewMode,
    pub configuration: Arc<Mutex<AppConfig>>, 
    pub ticket_provider: Arc<Mutex<TicketProvider>>,
    pub invalidate_cache: bool,
    bucket_panel: Vec<BucketPanelFolder>,
    open_folders: Rc<RefCell<Vec<String>>>,
    selected_filters: Rc<RefCell<Vec<BucketPanelLocation>>>,
    visible_tickets: Vec<Ticket>,
    overlay: Overlay,
    pub update_panel_data: Arc<Mutex<bool>>
}

impl UIController {

    pub fn new(configuration: Arc<Mutex<AppConfig>>, ticket_provider: Arc<Mutex<TicketProvider>>, update_trigger: Arc<Mutex<bool>>) -> Self {

        //show wizard, if config does not contain the wizard flag

        let mut overlay = Overlay::None;
        let mut ticket_view_mode = TicketViewMode::Regular;
        let mut show_sidebar = true;

        match configuration.lock() {
            Ok(mut lock) => {
                match lock.get_or_default("wizard", true, "").get::<bool>() {
                    Some(is_wizard_active) => {
                        if is_wizard_active {
                            overlay = Overlay::Wizard(WizardData::default())
                        }
                    },
                    None => (),
                };

                match lock.get_or_default("ticket:view_mode", "regular", "").raw().as_str() {
                    "regular" => ticket_view_mode = TicketViewMode::Regular,
                    "half" => ticket_view_mode = TicketViewMode::Half,
                    "list" => ticket_view_mode = TicketViewMode::List,
                    _ => ()
                }

                match lock.get_or_default("sidebar:enabled", true, "").get::<bool>() {
                    Some(sidebar_enabled) => {
                        show_sidebar = sidebar_enabled;
                    },
                    None => (),
                }
            },
            Err(err) => println!("Wasn't able to lock Configuration, due to {err}"),
        }

        let mut controller = UIController {
            font_changed: true, 
            running: true,
            show_sidebar: show_sidebar,
            ticket_view_mode: ticket_view_mode,
            configuration: configuration, 
            ticket_provider: ticket_provider,
            bucket_panel: vec![],
            selected_filters: Rc::new(RefCell::new(vec![])),
            visible_tickets: vec![],
            open_folders: Rc::new(RefCell::new(vec![])),
            invalidate_cache: true,
            overlay: overlay,
            update_panel_data: update_trigger,
        };

        controller.update_bucket_panel_data();

        controller
    }

    /**
       tells the UI Controller to invalidate the UI Cache. If the cache is given,
       the cache is invalidated immediately.
     */
    pub fn invalidate_cache(&mut self, cache: Option<&mut UICache>) {
        self.invalidate_cache = true;

        if let Some(cache) = cache {
            cache.refresh_cache(self)
        }
    }

    pub fn on_close_ui(&self, theme: &UITheme, frame: &mut eframe::Frame) {
        frame.close();
        //theme.write(self.configuration.clone());
    }

    pub fn read_image_data_from_path(&self, path: &Path) -> Option<ImageData> {
        
        match File::open(path) {
            Ok(handle) => {
                let decoder = png::Decoder::new(handle);
                let mut reader = decoder.read_info().unwrap();
                let mut buf = vec![0; reader.output_buffer_size()];
                let info = reader.next_frame(&mut buf).unwrap();
                let bytes = &buf[..info.buffer_size()];
        
                Some((bytes.to_vec(), info.width, info.height))
            },
            Err(err) => {
                println!("Couldn't open file, due to {err}");
                None
            },
        }
    }

    fn as_color(value: String) -> Option<Color32> {

        let subs = value[1..].as_bytes()
        .chunks(2)
        .map( |unmapped| u8::from_str_radix(std::str::from_utf8(unmapped).unwrap(), 16))
        .collect::<Result<Vec<u8>, _>>()
        .unwrap();

        if subs.len() >= 3 {
            Some(Color32::from_rgba_premultiplied(
                *subs.get(0).unwrap(), 
                *subs.get(1).unwrap(), 
                *subs.get(2).unwrap(), 
                *subs.get(3).unwrap_or(&255)
            ))
        } else {
            None
        }
    }

    pub fn color_as_string(value: Color32) -> String {
        let mut col_string = String::from("#");
        for byte in value.to_array() {
            write!(&mut col_string, "{:02X}", byte).expect("Unable to write string for converting color!");
        }

        col_string
    }

    pub fn using_ticket_provider(&self, with_ticket_provider: impl FnOnce(&UIController, &mut TicketProvider)) -> bool {
        let ticket_provider = self.ticket_provider.clone();
        let ticket_provider_lock = ticket_provider.lock();
        match ticket_provider_lock {
            Ok(mut ticket_provider) => {
                with_ticket_provider(self, &mut ticket_provider);
                true
            },
            Err(err) => {
                println!("Wasn't able to lock Ticket Provider due to {err}");
                false
            },
        }
    }

    pub fn using_ticket_provider_mut(&mut self, with_ticket_provider: impl FnOnce(&mut UIController, &mut TicketProvider)) -> bool {
        let ticket_provider = self.ticket_provider.clone();
        let ticket_provider_lock = ticket_provider.lock();
        match ticket_provider_lock {
            Ok(mut ticket_provider) => {
                with_ticket_provider(self, &mut ticket_provider);
                true
            },
            Err(err) => {
                println!("Wasn't able to lock Ticket Provider due to {err}");
                false
            },
        }
    }

    pub fn get_tags_cache(&self) -> TagsCache {
        let mut results: TagsCache = HashMap::new();
        match self.ticket_provider.lock() {
            Ok(lock) => {
                for tag in lock.tag_list_all() {
                    let color_opt = UIController::as_color(tag.color);
                    let color_text_opt = UIController::as_color(tag.color_text);

                    let color = match color_opt {
                        Some(col) => col,
                        None => Color32::BLACK,
                    };

                    let color_text = match color_text_opt {
                        Some(col) => col,
                        None => Color32::WHITE,
                    };

                    results.insert(TagCacheKey::new(tag.name, tag.adapter), [color, color_text]);
                }
            },
            Err(err) => println!("Wasn't able to open ticket provider for getting tag colors, {err}"),
        }

        results
    }

    pub fn get_current_overlay(&mut self) -> &mut Overlay {
        &mut self.overlay
    }

    pub fn has_overlay(&self) -> bool {
        self.overlay != Overlay::None
    }

    pub fn open_overlay(&mut self, new_overlay: Overlay) {
        self.overlay = new_overlay;
    }

    pub fn close_overlay(&mut self) {
        self.overlay = Overlay::None;
    }

    pub fn get_states(&self) -> HashMap<StateIdentifier, String> {
        let mut results: HashMap<StateIdentifier, String> = HashMap::new();

        match self.ticket_provider.lock() {
            Ok(lock) => {
                for state in lock.state_list_all() {
                    results.insert(state.identifier.clone(), state.description);
                }
            },
            Err(err) => println!("Wasn't able to open ticket provider for getting state descriptors, {err}"),
        }

        results
    }

    pub fn read_adapter_icons(&self, icons: &mut HashMap<String, Option<ColorImage>>) {

        match self.ticket_provider.lock() {
            Ok(lock) => {
                for adapter in lock.list_adapter_refs() {
                    self.read_custom_icon(icons, adapter.get_icon(), adapter.get_name());
                }
            },
            Err(err) => println!("Wasn't able to lock Ticket Provider for reading Icon Paths due to {err}"),
        }
    }

    pub fn read_custom_icon(&self, icons: &mut HashMap<String, Option<ColorImage>>, icon_path: &Path, identifier: String) {
        let image_data = self.read_image_data_from_path(icon_path);
        
        let color_image = match image_data {
            Some(found_data) => {
                let image = UIController::image_data_as_image(found_data);
                Some(image)
            },
            None => None,
        };
        icons.insert(identifier, color_image);
    }

    pub fn image_data_as_icon(image_data: ImageData) -> IconData {
        IconData { 
            rgba: image_data.0, 
            width: image_data.1, 
            height: image_data.2 
        }
    }

    pub fn image_data_as_image(image_data: ImageData) -> ColorImage {
        ColorImage::from_rgba_unmultiplied(
            [image_data.1 as usize, image_data.2 as usize], 
            image_data.0.as_slice()
        )
    }

    pub fn trigger_bucket_panel_update(&mut self) {
        if let Ok(mut lock) = self.update_panel_data.lock() {
            *lock = true;
        };
    }

    pub fn check_bucket_panel_trigger(&mut self, cache: &mut UICache) {

        let mut update = false;

        if let Ok(mut lock) = self.update_panel_data.lock() {
            if *lock {
                update = *lock;
                *lock = false;
            }
        };

        if update {
            self.update_bucket_panel_data();
            self.invalidate_cache(Some(cache));
        }
    }

    fn update_bucket_panel_data(&mut self) {

        match self.ticket_provider.lock() {
            Ok(lock) => {

                let mut adapters_index: BTreeMap<String, BucketPanelFolder> = BTreeMap::new();

                // Add a custom filters folder
                let mut custom_filters = BucketPanelFolder {
                    label: "Filters".to_string(),
                    adapter: "_custom_filters".to_string(),
                    is_open: false,
                    entries: vec![]
                };

                adapters_index.insert("_custom_filters".to_string(), custom_filters.clone());

                // Add the folders
                for adapter in lock.list_adapter_refs() {
                    let mut folder = BucketPanelFolder {
                        label: adapter.get_fancy_name(),
                        adapter: adapter.get_name(),
                        is_open: false,
                        entries: vec![]
                    };

                    for filter in adapter.filter_list_all() {

                        let mut filter_instance = BucketPanelEntry {
                            label: filter.identifier.name.clone(),
                            adapter: filter.identifier.adapter,
                            filter: filter.identifier.name,
                            entry_type: filter.filter_type,
                        };

                        match filter_instance.entry_type.clone() {
                            FilterType::User => custom_filters.entries.push(filter_instance),
                            FilterType::Builtin => folder.entries.push(filter_instance),
                            FilterType::Bucket(_) => folder.entries.push(filter_instance),
                            FilterType::Tag => folder.entries.push(filter_instance),
                            FilterType::Other => folder.entries.push(filter_instance),
                        };
                    }

                    adapters_index.insert(adapter.get_name(), folder);
                }

                adapters_index.insert("_custom_filters".to_string(), custom_filters);

                self.bucket_panel.clear();
                for entry in adapters_index {
                    self.bucket_panel.push(entry.1);
                }

            },
            Err(err) => println!("Wasn't able to lock TicketProvider! Reason is {err}"),
        };
    }

    pub fn update_each_ticket(
        &mut self, 
        ui: &mut Ui, 
        icon_textures: &mut HashMap<String, Option<TextureHandle>>, 
        icons: &mut HashMap<String, Option<ColorImage>>,
        theme: &UITheme,
        cache: &mut UICache
    ) {

        let mut action = TicketAction::None;

        let max_width = ui.available_width() - (theme.font_size as f32) * 1.5 ;

        for ticket in &self.visible_tickets {
            let ticket_icon = UserInterface::load_texture(icon_textures, icons, ui, &ticket.adapter);
            

            let temp_action = match self.ticket_view_mode {
                TicketViewMode::Regular => {
                    let temp_action = UserInterface::update_ticket_regular(ui, &ticket, theme, ticket_icon, cache, max_width);
                    ui.add_space(8.0);
                    temp_action
                },
                TicketViewMode::Half => {
                    let temp_action = UserInterface::update_ticket_half(ui, &ticket, theme, ticket_icon, cache, max_width);
                    ui.add_space(4.0);
                    temp_action
                },
                TicketViewMode::List => {
                    let temp_action = UserInterface::update_ticket_list(ui, &ticket, theme, ticket_icon, cache, max_width);
                    ui.add_space(2.0);
                    temp_action
                },
            };

            if temp_action != TicketAction::None {
                action = temp_action;
            }

            
        }

        match action {
            TicketAction::NewInBucket(id) => {

                let mut ticket: Ticket = Ticket::default();
                ticket.adapter = id.adapter;
                ticket.bucket_id = id.id;

                self.open_overlay(self.create_new_ticket_overlay(Some(ticket)));
            },

            TicketAction::Delete(id) => {
                self.using_ticket_provider_mut(|controller, provider| {

                    if let Some(ticket_ref) = provider.ticket_list_unique(id.id, &id.adapter) {
                        controller.open_overlay(Overlay::DeleteTicket(UpdateTicketData {
                            ticket: ticket_ref,
                            ..Default::default()
                        }));
                    }
                });
            },
            TicketAction::UpdateDetails(id) => {
                self.using_ticket_provider_mut(|controller, provider| {

                    if let Some(ticket_ref) = provider.ticket_list_unique(id.id, &id.adapter) {
                        controller.open_overlay(Overlay::UpdateTicketDetails(UpdateTicketData {
                            ticket: ticket_ref,
                            ..Default::default()
                        }));
                    }
                });
            },
            TicketAction::UpdateAssign(id) => {
                let mut ticket_ref: Option<Ticket> = None;
                self.using_ticket_provider(|controller, provider| {
                     ticket_ref = match provider.ticket_list_unique(id.id, &id.adapter) {
                        Some(mut ticket_ref) => {
                            Some(ticket_ref)
                        },
                        None => None,
                    };
                });

                if let Some(ticket) = ticket_ref {
                    self.open_overlay(self.create_ticket_assign_overlay(Some(ticket)));
                }
            },
            TicketAction::UpdateState(id) => {
                self.using_ticket_provider_mut(|controller, provider| {

                    if let Some(ticket_ref) = provider.ticket_list_unique(id.id, &id.adapter) {
                        controller.open_overlay(Overlay::UpdateTicketState(UpdateTicketData {
                            ticket: ticket_ref,
                            ..Default::default()
                        }));
                    }
                });
            },
            TicketAction::AddTagImmediate(id, tag) => {

                let mut ticket_ref: Option<Ticket> = None;
                self.using_ticket_provider(|controller, provider| {
                     ticket_ref = match provider.ticket_list_unique(id.id, &id.adapter) {
                        Some(mut ticket_ref) => {
                            ticket_ref.tags.push(tag);
                            Some(ticket_ref)
                        },
                        None => None,
                    };
                });

                if let Some(ticket) = ticket_ref {
                    OverlayAction::UpdateTicket(ticket).execute(self, cache);
                }

            },
            TicketAction::RemoveTagImmediate(id, tag) => {

                let mut ticket_ref: Option<Ticket> = None;
                self.using_ticket_provider(|controller, provider| {
                     ticket_ref = match provider.ticket_list_unique(id.id, &id.adapter) {
                        Some(mut ticket_ref) => {
                            ticket_ref.tags.retain(|curr_tag| tag.ne(curr_tag));
                            Some(ticket_ref)
                        },
                        None => None,
                    };
                });

                if let Some(ticket) = ticket_ref {
                    OverlayAction::UpdateTicket(ticket).execute(self, cache);
                }

            },
            TicketAction::NewTag(id) => {

                let mut tag = Tag::default()
                    .with_name(id.id)
                    .with_random_colors();
                tag.adapter = id.adapter;

                self.open_overlay(self.create_new_tag_overlay(Some(tag)));
            },
            TicketAction::Clone(id) => {
                let mut ticket_ref: Option<Ticket> = None;
                self.using_ticket_provider(|controller, provider| {
                     ticket_ref = match provider.ticket_list_unique(id.id, &id.adapter) {
                        Some(mut ticket_ref) => {
                            ticket_ref.id = 0;
                            ticket_ref.title += " (Clone)";
                            Some(ticket_ref)
                        },
                        None => None,
                    };
                });

                if let Some(ticket) = ticket_ref {
                    self.open_overlay(self.create_new_ticket_overlay(Some(ticket)));
                }
                
            },
            TicketAction::UpdateBucket(id) => {
                self.using_ticket_provider_mut(|controller, provider| {

                    let buckets: Vec<Bucket> = vec![];
                    if let Some(ticket_ref) = provider.ticket_list_unique(id.id, &id.adapter) {
                        controller.open_overlay(Overlay::UpdateTicketBucket(UpdateTicketDataBucket {
                            buckets: provider.bucket_list_all(),
                            ticket: ticket_ref,
                            ..Default::default()
                        }));
                    }
                });
            },
            TicketAction::UpdateAdapter(id) => {
                let mut ticket_ref: Option<Ticket> = None;
                self.using_ticket_provider(|_, provider| {
                     ticket_ref = provider.ticket_list_unique(id.id, &id.adapter);
                });

                if let Some(ticket) = ticket_ref {
                    self.open_overlay(self.create_edit_ticket_adapter_overlay(ticket));
                }
            },
            TicketAction::None => (),
            TicketAction::UpdateStateImmediate(id, state) => {

                let mut ticket_ref: Option<Ticket> = None;
                self.using_ticket_provider(|controller, provider| {
                     ticket_ref = match provider.ticket_list_unique(id.id, &id.adapter) {
                        Some(mut ticket_ref) => {
                            ticket_ref.state_name = state;
                            Some(ticket_ref)
                        },
                        None => None,
                    };
                });

                if let Some(ticket) = ticket_ref {
                    OverlayAction::UpdateTicket(ticket).execute(self, cache);
                }
            },
            TicketAction::DropTagImmediate(tag_ref) => {
                OverlayAction::DeleteTag(tag_ref).execute(self, cache);
            },
            TicketAction::Edit(id) => {
                let mut ticket_ref: Option<Ticket> = None;
                self.using_ticket_provider(|_, provider| {
                     ticket_ref = provider.ticket_list_unique(id.id, &id.adapter);
                });

                if let Some(ticket) = ticket_ref {
                    self.open_overlay(self.create_edit_ticket_overlay(Some(ticket)));
                }
            },
        };

    }

    pub fn create_ticket_assign_overlay(&self, ticket: Option<Ticket>) -> Overlay {
        let ticket = match ticket {
            Some(ticket) => ticket,
            None => Ticket::default(),
        };

        let mut username = String::from("new User");
        match self.configuration.lock() {
            Ok(mut lock) => {
                username = lock.get_or_default("username", "new User", "").raw().clone();
            },
            Err(err) => println!("Wasn't able to lock configuration due to {}", err),
        };

        Overlay::UpdateTicketAssign(UpdateTicketDataAssign{
            username,
            ticket,
            ..Default::default()
        })
    }

    pub fn create_edit_ticket_overlay(&self, ticket: Option<Ticket>) -> Overlay {
        let ticket = match ticket {
            Some(ticket) => ticket,
            None => Ticket::default(),
        };

        let mut username = String::from("new User");
        match self.configuration.lock() {
            Ok(mut lock) => {
                username = lock.get_or_default("username", "new User", "").raw().clone();
            },
            Err(err) => println!("Wasn't able to lock configuration due to {}", err),
        };

        let mut buckets: Vec<Bucket> = vec![];
        self.using_ticket_provider(|_, provider| {
            buckets = provider.bucket_list_all();
        });

        let due_date = match Utc.timestamp_millis_opt(ticket.due_at) {
            chrono::LocalResult::None => Utc::now(),
            chrono::LocalResult::Single(result) => result,
            chrono::LocalResult::Ambiguous(_, result) => result,
        };

        Overlay::EditTicket(EditTicketData{
            username,
            buckets,
            ticket,
            due_date,
            ..Default::default()
        })
    }

    pub fn create_edit_ticket_adapter_overlay(&self, ticket: Ticket) -> Overlay {

        let mut adapters: Vec<(String, String)> = vec![];
        self.using_ticket_provider(|_, provider| {
            adapters = provider.list_adapter_name_pairs();
        });

        Overlay::UpdateTicketAdapter(UpdateTicketDataAdapter{
            old_adapter: ticket.adapter.clone(),
            adapters,
            ticket,
            ..Default::default()
        })
    }

    pub fn create_new_ticket_overlay(&self, ticket: Option<Ticket>) -> Overlay {

        let ticket = match ticket {
            Some(ticket) => ticket,
            None => Ticket::default(),
        };

        let mut username = String::from("new User");
        match self.configuration.lock() {
            Ok(mut lock) => {
                username = lock.get_or_default("username", "new User", "").raw().clone();
            },
            Err(err) => println!("Wasn't able to lock configuration due to {}", err),
        };

        let mut buckets: Vec<Bucket> = vec![];
        let mut adapters: Vec<(String, String)> = vec![];
        self.using_ticket_provider(|_, provider| {
            buckets = provider.bucket_list_all();
            adapters = provider.list_adapter_name_pairs();
        });

        let due_date = match Utc.timestamp_millis_opt(ticket.due_at) {
            chrono::LocalResult::None => Utc::now(),
            chrono::LocalResult::Single(result) => result,
            chrono::LocalResult::Ambiguous(_, result) => result,
        };

        Overlay::NewTicket(NewTicketData{
            username,
            buckets,
            adapters,
            ticket,
            due_date,
            ..Default::default()
        })

    }

    pub fn create_preferences_overlay(&self) -> Overlay {
        let mut username = "New User".to_string();

        match self.configuration.lock() {
            Ok(mut lock) => username = lock.get_or_default("username", "New User", "").raw().clone(),
            Err(err) => println!("Wasn't able to lock App config for preferences, due to {err}"),
        }

        Overlay::Preferences(PreferenceData{
            username,
            ..Default::default()
        })
    }

    pub fn create_new_bucket_overlay(&self, bucket: Option<Bucket>) -> Overlay {
        let bucket = match bucket {
            Some(bucket) => bucket,
            None => Bucket::default()
        };

        let mut adapters: Vec<(String, String)> = vec![];
        match self.ticket_provider.lock() {
            Ok(mut lock) => {
                adapters = lock.list_adapter_name_pairs();
            },
            Err(err) => println!("Wasn't able to lock ticket provider due to {}", err)
        }

        Overlay::NewBucket(NewBucketData{
            bucket: bucket,
            adapters: adapters,
            ..Default::default()
        })
    }

    pub fn create_new_tag_overlay(&self, tag: Option<Tag>) -> Overlay {

        let tag = match tag {
            Some(tag) => tag,
            None => Tag::default().with_random_colors(),
        };

        let color = match UIController::as_color(tag.color.clone()) {
            Some(col) => col,
            None => Color32::DARK_GRAY,
        };

        let color_text = match UIController::as_color(tag.color_text.clone()) {
            Some(col) => col,
            None => Color32::WHITE,
        };

        let mut adapters: Vec<(String, String)> = vec![];
        match self.ticket_provider.lock() {
            Ok(mut lock) => {
                adapters = lock.list_adapter_name_pairs();
            },
            Err(err) => println!("Wasn't able to lock ticket provider due to {}", err)
        }

        Overlay::NewTag(NewTagData{
            tag: tag,
            back_color: color,
            font_color: color_text,
            adapters: adapters,
            ..Default::default()
        })

    }

    pub fn create_new_state_overlay(&self, state: Option<State>) -> Overlay {
        
        let state = match state {
            Some(state) => state,
            None => State::default(),
        };

        let mut adapters: Vec<(String, String)> = vec![];
        self.using_ticket_provider(|_, provider| {
            adapters = provider.list_adapter_name_pairs();
        });

        Overlay::NewState(NewStateData{
            state: state,
            adapters: adapters,
            ..Default::default()
        })
    }

    pub fn update_each_folder(&mut self, ui: &mut Ui, ui_theme: &UITheme) -> Option<BucketPanelLocation> {
        
        let mut found: Option<BucketPanelLocation> = None;
        let mut overlay = Overlay::None;
        
        for folder in &self.bucket_panel {

            let folder_open = self.is_folder_open(&folder.adapter);
            let folder_in_panel = self.is_folder_in_panel(&folder.adapter);
            let is_filter = folder.adapter.eq("_custom_filters");

            match UserInterface::update_side_panel_folder(ui, ui_theme, folder_in_panel, folder_open, !is_filter, folder) {
                SidePanelAction::FolderClicked => {
                    
                    found = Some(BucketPanelLocation { 
                        entry_type: BucketPanelLocationType::Adapter, 
                        adapter: folder.adapter.clone(), 
                        section: "".to_string() 
                    });

                }

                SidePanelAction::FolderOpenClose => self.open_close_folder_in_panel(&folder.adapter),
                SidePanelAction::FolderNewTag => {
                    let mut tag = Tag{
                        adapter: folder.adapter.clone(),
                        ..Default::default()
                    }.with_random_colors();

                    overlay = self.create_new_tag_overlay(Some(tag))
                },
                SidePanelAction::FolderNewBucket => {
                    let mut bucket = Bucket{
                        identifier: BucketIdentifier{
                            adapter: folder.adapter.clone(),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    overlay = self.create_new_bucket_overlay(Some(bucket))
                },
                SidePanelAction::FolderNewFilter => {

                    let mut adapters: Vec<(String, String)> = vec![];
                    self.using_ticket_provider(|_, provider| {
                        adapters = provider.list_adapter_name_pairs();
                    });

                    overlay = Overlay::NewFilter(NewFilterData {
                        filter: Filter{
                            identifier: FilterIdentifier {
                                adapter: folder.adapter.clone(),
                                name: String::default(),
                            },
                            operation: "[[...]]".to_string(),
                            filter_type: FilterType::User,
                        },
                        adapters,
                        ..Default::default()
                    });
                },
                SidePanelAction::FolderNewState => {
                    let mut state = State{
                        identifier: StateIdentifier {
                            adapter: folder.adapter.clone(),
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    overlay = self.create_new_state_overlay(Some(state))
                },
                SidePanelAction::FolderNewTicket => {
                    let mut ticket = Ticket{
                        adapter: folder.adapter.clone(),
                        ..Default::default()
                    };
                    overlay = self.create_new_ticket_overlay(Some(ticket))
                },
                SidePanelAction::FolderRemove => {
                    overlay = Overlay::DeleteAdapter(DeleteAdapterData{
                        adapter_name: folder.adapter.clone(),
                    });
                },
                _ => (),
                
            }

            if folder_open {
                for entry in &folder.entries {

                    let entry_in_panel = self.is_entry_in_panel(&entry.adapter, &entry.label);

                    match UserInterface::update_side_panel_entry(ui, ui_theme, entry_in_panel || folder_in_panel, entry) {
                        SidePanelAction::EntryClicked => {
                            found = Some(BucketPanelLocation { 
                                entry_type: if is_filter { 
                                    BucketPanelLocationType::Filter
                                } else { 
                                    BucketPanelLocationType::Entry 
                                }, 
                                adapter: entry.adapter.clone(),
                                section: entry.label.clone() 
                            });
                        },
                        SidePanelAction::EntryRemove => {

                            self.using_ticket_provider(|_, provider| {
                                if let Some(filter) = provider.filter_list_unique(&entry.filter, &entry.adapter) {
                                    overlay = Overlay::DeleteFilter(DeleteFilterData {
                                        filter,
                                        ..Default::default()
                                    });
                                }
                            });
                        },
                        SidePanelAction::EntryEdit => {
                            self.using_ticket_provider(|_, provider| {
                                if let Some(filter) = provider.filter_list_unique(&entry.filter, &entry.adapter) {
                                    overlay = Overlay::EditFilter(EditFilterData {
                                        filter,
                                        ..Default::default()
                                    });
                                }
                            });
                        },
                        SidePanelAction::EntryBucketRemove(id) => {
                            self.using_ticket_provider(|_, provider| {
                                if let Some(bucket) = provider.bucket_list_unique(id, &entry.adapter) {
                                    overlay = Overlay::DeleteBucket(DeleteBucketData {
                                        bucket,
                                        ..Default::default()
                                    });
                                }
                            });
                        }
                        _ => (),
                    }
                };
            };
        };

        if overlay != Overlay::None {
            self.open_overlay(overlay);
        }

        if UserInterface::update_side_panel_space(ui) {
            found = Some(BucketPanelLocation {
                entry_type: BucketPanelLocationType::Reset,
                adapter: "".to_string(),
                section: "".to_string()
            })
        }

        return found;
    }

    pub fn is_folder_in_panel(&self, adapter: &String) -> bool {
        self.selected_filters.borrow().iter().any(|selection| {
            &selection.adapter == adapter &&
            selection.entry_type == BucketPanelLocationType::Adapter
        })
    }

    pub fn is_entry_in_panel(&self, adapter: &String, entry: &String) -> bool {
        self.selected_filters.borrow().iter().any(|selection| {
            &selection.adapter == adapter &&
            &selection.section == entry &&
            (selection.entry_type == BucketPanelLocationType::Entry ||
            selection.entry_type == BucketPanelLocationType::Filter)
        })
    }

    pub fn is_folder_open(&self, label: &String) -> bool {
        self.open_folders.borrow().iter().any(|open_folder| open_folder.eq(label))
    }

    pub fn open_close_folder_in_panel(&self, label: &String) {

        let open_folders = self.open_folders.clone();

        match open_folders.try_borrow_mut() {
            Ok(mut borrow_mut) => {

                match borrow_mut.iter().position(|open_folder| open_folder.eq(label)) {
                    Some(index) => {
                        borrow_mut.remove(index);
                    },
                    None => {
                        borrow_mut.push(label.clone());
                    },
                }

            },
            Err(err) => println!("Failed to mutable borrow open_folders, due to {}", err),
        };
    }

    pub fn toggle_folder_in_panel(&self, folder: BucketPanelLocation, modifier_button: bool) {

        // If shift or ctrl has not been pressed
        if !modifier_button {
            let filters = self.selected_filters.clone();

            filters.borrow_mut().clear();
            filters.borrow_mut().push(folder);

        } else if !self.is_folder_in_panel(&folder.adapter) {
            let filters = self.selected_filters.clone();
            filters.borrow_mut().push(folder);
        }
    }

    pub fn execute_bucket_panel_selection(&mut self) {

        self.using_ticket_provider_mut(|controller, provider| {
            let tickets = provider.ticket_list_from_selection(&controller.selected_filters.borrow());

            if let Some(mut tickets) = tickets {
                controller.visible_tickets.clear();
                controller.visible_tickets.append(&mut tickets);
            };
        });
    }

}