use std::sync::{Arc, Mutex};
use eframe::egui::Color32;
use eframe::Theme;
use egui::{hex_color, epaint};
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
            background_primary: Color32::from_rgba_unmultiplied(0x1a, 0x1a, 0x1a, 0xff),
            background_secondary: Color32::from_rgba_unmultiplied(0x25, 0x25, 0x25, 0xff),
            background_tertiary: Color32::from_rgba_unmultiplied(0x2d, 0x2d, 0x2d, 0xff),
            background_error: Color32::from_rgba_unmultiplied(0x38, 0x18, 0x18, 0xff),
            foreground_primary: Color32::from_rgba_unmultiplied(0xcc, 0xcc, 0xcc, 0xff),
            foreground_secondary: Color32::from_rgba_unmultiplied(0x80, 0x80, 0x80, 0xff),
            foreground_tertiary: Color32::from_rgba_unmultiplied(0xf5, 0xf5, 0xf5, 0xff),
            foreground_marker: Color32::from_rgba_unmultiplied(0x89, 0x66, 0x10, 0xff),
            foreground_marker2: Color32::from_rgba_unmultiplied(0xda, 0xa5, 0x20, 0xff),
            font_size: 13,
            config: None,
        }
    }

    pub fn theme_light() -> UITheme {
        UITheme {
            base_theme: Theme::Light,
            background_primary: Color32::from_rgba_unmultiplied(0xe0, 0xe0, 0xe0, 0xff),
            background_secondary: Color32::from_rgba_unmultiplied(0xb8, 0xb8, 0xb8, 0xff),
            background_tertiary: Color32::from_rgba_unmultiplied(0x92, 0x92, 0x92, 0xff),
            background_error: Color32::from_rgba_unmultiplied(0xfb, 0x98, 0x97, 0xff),
            foreground_primary: Color32::from_rgba_unmultiplied(0x2d, 0x2d, 0x2d, 0xff),
            foreground_secondary: Color32::from_rgba_unmultiplied(0x51, 0x51, 0x51, 0xff),
            foreground_tertiary: Color32::from_rgba_unmultiplied(0x13, 0x13, 0x13, 0xff),
            foreground_marker: Color32::from_rgba_unmultiplied(0x66, 0x4f, 0x17, 0xff),
            foreground_marker2: Color32::from_rgba_unmultiplied(0x2b, 0x20, 0x04, 0xff),
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