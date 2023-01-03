use std::{sync::{Arc, Mutex}};
use egui::Color32;
use egui::epaint::hex_color;
use eframe::Theme;
use tickets_rs_core::AppConfig;

#[derive(Clone)]
pub struct UITheme {
    pub base_theme: Theme,
    pub background_primary: Color32,
    pub background_secondary: Color32,
    pub background_tertiary: Color32,
    pub background_error: Color32,
    pub foreground_primary: Color32,
    pub foreground_secondary: Color32,
    pub foreground_tertiary: Color32,
    pub foreground_marker: Color32,
    pub foreground_marker2: Color32,
    pub font_size: i32,
    pub config: Option<Arc<Mutex<AppConfig>>>,
}

impl PartialEq for UITheme {
    fn eq(&self, other: &Self) -> bool {
        self.base_theme == other.base_theme &&

        self.background_primary == other.background_primary &&
        self.background_secondary == other.background_secondary &&
        self.background_tertiary == other.background_tertiary &&
        self.background_error == other.background_error &&

        self.foreground_primary == other.foreground_primary &&
        self.foreground_secondary == other.foreground_secondary &&
        self.foreground_tertiary == other.foreground_tertiary &&
        self.foreground_marker == other.foreground_marker &&
        self.foreground_marker2 == other.foreground_marker2
    }
}

impl Default for UITheme {
    fn default() -> Self {
        UITheme::theme_dark()
    }
}

impl Drop for UITheme {
    fn drop(&mut self) {

        match &self.config {
            Some(config_mutex) => {
                self.write(config_mutex.clone())
            },
            None => (),
        };
    }
}

impl UITheme {

    pub fn theme_dark() -> UITheme {
        UITheme {
            base_theme: Theme::Dark,
            background_primary: hex_color!("#1a1a1aff"),
            background_secondary: hex_color!("#252525ff"),
            background_tertiary: hex_color!("#2d2d2dff"),
            background_error: hex_color!("#381818ff"),
            foreground_primary: hex_color!("#ccccccff"),
            foreground_secondary: hex_color!("#808080ff"),
            foreground_tertiary: hex_color!("#f5f5f5ff"),
            foreground_marker: hex_color!("#896610ff"),
            foreground_marker2: hex_color!("#daa520ff"),
            font_size: 13,
            config: None,
        }
    }

    pub fn theme_light() -> UITheme {
        UITheme {
            base_theme: Theme::Light,
            background_primary: hex_color!("#e0e0e0ff"),
            background_secondary: hex_color!("#b8b8b8ff"),
            background_tertiary: hex_color!("#929292ff"),
            background_error: hex_color!("#fb9897ff"),
            foreground_primary: hex_color!("#2d2d2dff"),
            foreground_secondary: hex_color!("#515151ff"),
            foreground_tertiary: hex_color!("#131313ff"),
            foreground_marker: hex_color!("#664f17ff"),
            foreground_marker2: hex_color!("#2b2004ff"),
            font_size: 13,
            config: None,
        }
    }

    pub fn name(&self) -> String {

        if *self == UITheme::theme_light() {
            return "Light Theme".to_string();
        }

        if *self == UITheme::theme_dark() {
            return "Dark Theme".to_string();
        }

        "Custom Theme".to_string()
    }

    pub fn names() -> Vec<String> {
        vec![
            "Light Theme".to_string(),
            "Dark Theme".to_string()
        ]
    }

    pub fn from_name(name: &str) -> UITheme {

        if name == "Light Theme" {
            return UITheme::theme_light();
        }

        if name == "Dark Theme" {
            return UITheme::theme_dark();
        }

        UITheme::default()
    }

    pub fn with_config(mut self, config: Option<Arc<Mutex<AppConfig>>>) -> Self {
        self.config = config;
        self
    }

    pub fn from(config_mutex: Arc<Mutex<AppConfig>>) -> UITheme {

        match config_mutex.lock() {
            Ok(mut config) => {
                UITheme {

                    config: Some(config_mutex.clone()),

                    base_theme: {
                        if config.get_or_default(
                            "theme:base", 
                            "dark", "").raw().to_ascii_lowercase() == *"dark" {
                                Theme::Dark
                            } else {
                                Theme::Light
                            }
                    },

                    background_primary: config.get_or_default( 
                        "color:background:primary", 
                        "#1a1a1aff", 
                        "").get().unwrap(),
        
                    background_secondary: config.get_or_default(
                        "color:background:secondary", 
                        "#252525ff", 
                        "").get().unwrap(),
        
                    background_tertiary: config.get_or_default(
                        "color:background:tertiary", 
                        "#2d2d2dff", 
                        "").get().unwrap(),

                    background_error: config.get_or_default(
                        "color:background:error", 
                        "#381818ff", 
                        "").get().unwrap(),
        
                    foreground_primary: config.get_or_default(
                        "color:foreground:primary", 
                        "#ccccccff", 
                        "").get().unwrap(),
        
                    foreground_secondary: config.get_or_default(
                        "color:foreground:secondary", 
                        "#808080ff", 
                        "").get().unwrap(),
        
                    foreground_tertiary: config.get_or_default(
                        "color:foreground:tertiary", 
                        "#f5f5f5ff", 
                        "").get().unwrap(),
        
                    foreground_marker: config.get_or_default(
                        "color:foreground:marker", 
                        "#896610ff", 
                        "").get().unwrap(),
        
                    foreground_marker2: config.get_or_default(
                        "color:foreground:marker2", 
                        "#daa520ff", 
                        "").get().unwrap(),
                    
                    font_size: config.get_or_default(
                        "font:size", 
                        13, 
                        "").get().unwrap()
                }
            },
            Err(err) => {
                println!("Wasn't able to lock configuration! reason: {}", err);
                UITheme::theme_dark()
            },
        }
    }

    pub fn get(&self, name: String) -> Color32 {
        match name.as_str() {
            "color:background:primary" => self.background_primary,
            "color:background:secondary" => self.background_secondary,
            "color:background:tertiary" => self.background_tertiary,
            "color:background:error" => self.background_error,
            "color:foreground:primary" => self.foreground_primary,
            "color:foreground:secondary" => self.foreground_secondary,
            "color:foreground:tertiary" => self.foreground_tertiary,
            "color:foreground:marker" => self.foreground_marker,
            "color:foreground:marker2" => self.foreground_marker2,
            _ => Color32::WHITE
        }
    }

    pub fn merge_colors(&mut self, other_theme: &UITheme) {
        self.base_theme = other_theme.base_theme;
        self.background_primary = other_theme.background_primary;
        self.background_secondary = other_theme.background_secondary;
        self.background_tertiary = other_theme.background_tertiary;
        self.background_error = other_theme.background_error;
        self.foreground_primary = other_theme.foreground_primary;
        self.foreground_secondary = other_theme.foreground_secondary;
        self.foreground_tertiary = other_theme.foreground_tertiary;
        self.foreground_marker = other_theme.foreground_marker;
        self.foreground_marker2 = other_theme.foreground_marker2;
    }

    fn write(&self, config: Arc<Mutex<AppConfig>>) {
        match config.lock() {
            Ok(mut lock) => {
                let base_theme_name = if self.base_theme == Theme::Dark {"dark"} else {"light"};
                lock.put("theme:base", base_theme_name, "");

                lock.put("color:background:primary", self.background_primary, "");
                lock.put("color:background:secondary", self.background_secondary, "");
                lock.put("color:background:tertiary", self.background_tertiary, "");
                lock.put("color:background:error", self.background_error, "");

                lock.put("color:foreground:primary", self.foreground_primary, "");
                lock.put("color:foreground:secondary", self.foreground_secondary, "");
                lock.put("color:foreground:tertiary", self.foreground_tertiary, "");
                lock.put("color:foreground:marker", self.foreground_marker, "");
                lock.put("color:foreground:marker2", self.foreground_marker2, "");

                lock.put("font:size", self.font_size, "");

            },
            Err(err) => println!("Wasn't able to lock Configuration, due to {}", err),
        }
    }
}