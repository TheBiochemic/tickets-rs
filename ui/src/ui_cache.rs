use std::collections::HashMap;
use egui::Color32;
use egui_commonmark::CommonMarkCache;
use tickets_rs_core::StateIdentifier;

use crate::UIController;

#[derive(Default, PartialEq, Eq, Hash)]
pub struct TagCacheKey {
    pub name: String,
    pub adapter: String
}

impl TagCacheKey {
    pub fn new(name: String, adapter: String) -> TagCacheKey {
        TagCacheKey { name: name, adapter: adapter }
    }
}

pub type TagCacheValue = [Color32; 2];
pub type TagsCache = HashMap<TagCacheKey, TagCacheValue>;



#[derive(Default)]
pub struct UICache {
    pub tags_valid: bool,
    pub tags: TagsCache,

    pub states_valid: bool,
    pub states: HashMap<StateIdentifier, String>,

    pub username_valid: bool,
    pub username: String,

    pub commonmark: CommonMarkCache
}

impl UICache {

    pub fn refresh_cache(&mut self, ui_controller: &mut UIController) {

        if ui_controller.invalidate_cache {
            self.tags_valid = false;
            self.states_valid = false;
        }

        self.refresh_tags(ui_controller);
        self.refresh_states(ui_controller);
        self.refresh_username(ui_controller);

        ui_controller.invalidate_cache = false;
    }

    pub fn refresh_username(&mut self, ui_controller: &UIController) {
        if !self.username_valid {
            self.username = match ui_controller.configuration.lock() {
                Ok(mut config) => config.get_or_default("username", "New User", "").raw().clone(),
                Err(err) => {
                    println!("Wasn't able to lock Configuration while refreshing username, due to {err}");
                    self.username.clone()
                },
            };
            self.username_valid = true;
        }
    }

    pub fn refresh_tags(&mut self, ui_controller: &UIController) {
        if !self.tags_valid {
            self.tags = ui_controller.get_tags_cache();
            self.tags_valid = true;
        }
    }

    pub fn refresh_states(&mut self, ui_controller: &UIController) {
        if !self.states_valid {
            self.states = ui_controller.get_states();
            self.states_valid = true;
        }
    }
}