use std::str::FromStr;
use egui::Color32;
use std::fmt::Write;
use std::str;

#[derive(Default, PartialEq, Clone)]
pub struct ConfigOption {
    pub(super) value: String,
    pub(super) display_options: String
}

pub trait ToConfig {
    fn to_config(&self) -> String;
    fn to_self(value: &str) -> Option<Self> where Self: Sized;
}

impl ToConfig for String {
    fn to_config(&self) -> String {
        self.to_string()
    }

    fn to_self(value: &str) -> Option<Self> {
        Some(value.to_string())
    }
}

impl ToConfig for &str {
    fn to_config(&self) -> String {
        self.to_string()
    }

    fn to_self(_value: &str) -> Option<Self> {
        None
    }
}

impl ToConfig for i32 {
    fn to_config(&self) -> String {
        self.to_string()
    }

    fn to_self(value: &str) -> Option<Self> {
        match i32::from_str(value) {
            Ok(int) => Some(int),
            Err(_) => None,
        }
    }
}

impl ToConfig for Color32 {
    fn to_config(&self) -> String {
        let mut col_string = String::from("#");
        for byte in self.to_array() {
            write!(&mut col_string, "{:02X}", byte).expect("Unable to write string for converting color!");
        }

        col_string
    }

    fn to_self(value: &str) -> Option<Self> {
        let subs = value[1..].as_bytes()
        .chunks(2)
        .map( |unmapped| u8::from_str_radix(str::from_utf8(unmapped).unwrap(), 16))
        .collect::<Result<Vec<u8>, _>>()
        .unwrap();

        if subs.len() >= 3 {
            Some(Color32::from_rgba_premultiplied(
                *subs.first().unwrap(), 
                *subs.get(1).unwrap(), 
                *subs.get(2).unwrap(), 
                *subs.get(3).unwrap_or(&255)
            ))
        } else {
            None
        }
    }
}

impl ToConfig for Vec<String> {
    fn to_config(&self) -> String {
        self.join("|||")
    }

    fn to_self(value: &str) -> Option<Self> where Self: Sized {
        let mut result_vec: Vec<String> = vec![];
        for value in value.split("|||") {
            result_vec.push(value.to_string());
        };

        Some(result_vec)
    }
}

impl ToConfig for bool {
    fn to_config(&self) -> String {
        if *self {
            "true".to_string()
        } else {
            "false".to_string()
        }
    }

    fn to_self(value: &str) -> Option<Self> where Self: Sized {
        match value {
            "true" => Some(true),
            "false" => Some(false),
            _ => None
        }
    }
}

impl ToConfig for Vec<&str> {
    fn to_config(&self) -> String {
        self.join("|||")
    }

    fn to_self(_value: &str) -> Option<Self> where Self: Sized {
        None
    }
}

impl ConfigOption {

    pub fn get<T: ToConfig>(&self) -> Option<T> {
        T::to_self(&self.value)
    }

    pub fn raw(&self) -> &String {
        &self.value
    }

    pub fn display_options(&self) -> &String {
        &self.display_options
    }
}