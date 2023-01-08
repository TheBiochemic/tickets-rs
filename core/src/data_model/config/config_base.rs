use std::collections::BTreeMap;

use super::{config_option::ToConfig, ConfigOption};

#[derive(Default, PartialEq, Clone)]
pub struct Config {
    options: BTreeMap<String, ConfigOption>,
}

impl Config {
    pub fn len(&self) -> usize {
        self.options.len()
    }

    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &ConfigOption)> {
        self.options.iter()
    }

    pub fn get(&self, name: &str) -> Option<&ConfigOption> {
        self.options.get(&name.to_string())
    }

    pub fn get_or_default<T: ToConfig>(&mut self, name: &str, value: T, display_options: &str) -> &ConfigOption {

        let name_string = name.to_string();

        if !self.options.contains_key(&name_string) {
            self.put(name, value, display_options);
        }

        self.options.get(name).unwrap()
    }

    pub fn put<T: ToConfig>(&mut self, name: &str, value: T, display_options: &str) {
        self.options.insert(
            name.to_string(), 
            ConfigOption {
                value: value.to_config(), 
                display_options: display_options.to_string(),
            });
    }

    pub fn drop_entry(&mut self, name: &str) -> bool {
        let name_string = name.to_string();
        let contains = self.options.contains_key(&name_string);
        if contains {
            self.options.remove(&name_string);
            true
        } else {
            false
        }
    }

    pub fn drop_sub_config(&mut self, prefix: &str) -> u32 {
        let mut found_names: Vec<String> = Vec::default();
        let prefix_string = prefix.to_string() + ":";
        let prefix_corrected = prefix_string.as_str();
        let mut removed_elements: u32 = 0;

        for entry in self.options.keys() {
            if entry.starts_with(prefix_corrected) {
                found_names.push(entry.clone())
            }
        }

        for to_remove in found_names {
            if self.options.remove(&to_remove).is_some() {
                removed_elements += 1;
            };
        }

        removed_elements
    }

    pub fn with<T: ToConfig>(mut self, name: &str, value: T, display_options: &str) -> Self {
        self.put(name, value, display_options);
        self
    }

    pub fn get_sub_config(&self, prefix: &str) -> Config {
        let mut sub_config = Config::default();

        for entry in &self.options {
            if entry.0.starts_with((prefix.to_string() + ":").as_str()) {

                let mut entry_name = entry.0.clone();
                entry_name = entry_name[prefix.len() + 1 .. entry_name.len()].to_string();

                sub_config.put(entry_name.as_str(), entry.1.value.clone(), &entry.1.display_options)
            }
        };

        sub_config
    }

    pub fn put_sub_config(&mut self, other: &Config, prefix: &str) {

        for entry in &other.options {
            let entry_name = prefix.to_string() + ":" + entry.0;
            self.put(entry_name.as_str(), entry.1.value.as_str(), entry.1.display_options.as_str());
        }

    }
}