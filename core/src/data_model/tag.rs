use rand::prelude::*;
use std::{fmt::Write, ops::Range};
use crate::TicketAdapter;

#[derive(Eq, PartialOrd, Ord, Debug, PartialEq, Clone)]
pub struct Tag {
    pub adapter: String,
    pub name: String,
    pub color: String,
    pub color_text: String
}

impl Default for Tag {
    fn default() -> Self {
        Tag {
            adapter: String::default(),
            name: String::default(),
            color: String::from("#ffffffff"),
            color_text: String::from("#000000ff")
        }
    }
}

impl Tag {

    pub fn generate_random_color(
        hue_range: Range<f32>, 
        sat_range: Range<f32>, 
        val_range: Range<f32>) -> (f32, f32, f32) {
        let mut rng = rand::thread_rng();
        let (hue, sat, val);

        hue = rng.gen_range(hue_range);
        sat = rng.gen_range(sat_range);
        val = rng.gen_range(val_range);

        (hue, sat, val)
    }

    pub fn with_random_colors(mut self) -> Self {
        let mut rng = rand::thread_rng();
        let (hue, sat, val);
        let (hue_text, sat_text, val_text);
        let choice: u8 = rng.gen_range(1..=6);

        match choice {
            1 => {
                //Colorful background with dark Text
                hue = rng.gen_range(0.0 .. 1.0);
                sat = rng.gen_range(0.5 ..= 1.0);
                val = 0.9;
                hue_text = hue;
                sat_text = sat;
                val_text = 0.15;
            },
            2 => {
                //light colored Background with Colored Text
                hue = rng.gen_range(0.0 .. 1.0);
                sat = rng.gen_range(0.0 ..= 0.25);
                val = 0.9;
                hue_text = hue;
                sat_text = 1.0;
                val_text = 0.8;
            },
            3 => {
                //dark colored Background with light text
                hue = rng.gen_range(0.0 .. 1.0);
                sat = rng.gen_range(0.5 ..= 1.0);
                val = 0.15;
                hue_text = hue;
                sat_text = sat;
                val_text = 0.9;
            },
            4 => {
                //gray background with black or white text
                hue = 0.0;
                sat = 0.0;
                val = rng.gen_range(0.2 ..= 0.8);
                hue_text = hue;
                sat_text = sat;
                val_text = if val > 0.5 { val - 0.5 } else { val + 0.5 };
            },
            5 => {
                //colorless dark background with colored text
                hue = 0.0;
                sat = 0.0;
                val = rng.gen_range(0.2 ..= 0.4);

                hue_text = rng.gen_range(0.0 .. 1.0);
                sat_text = rng.gen_range(0.5 ..= 1.0);
                val_text = rng.gen_range(0.6 ..= 1.0);
            },
            _ => {
                //colorless light background with colored text
                hue = 0.0;
                sat = 0.0;
                val = rng.gen_range(0.6 ..= 0.8);

                hue_text = rng.gen_range(0.0 .. 1.0);
                sat_text = rng.gen_range(0.5 ..= 1.0);
                val_text = rng.gen_range(0.0 ..= 0.4);
            }
        }

        let (r, g, b) = Tag::hsv_to_rgb(hue, sat, val);
        let (rt, gt, bt) = Tag::hsv_to_rgb(hue_text, sat_text, val_text);

        let mut color_string = "#".to_string();
        write!(&mut color_string, "{:02x}{:02x}{:02x}{:02x}", r, g, b, 255).expect("Failed to create Color string in Tag");
        self.color =  color_string;

        let mut color_text_string = "#".to_string();
        write!(&mut color_text_string, "{:02x}{:02x}{:02x}{:02x}", rt, gt, bt, 255).expect("Failed to create Text Color string in Tag");
        self.color_text =  color_text_string;
        self
    }

    pub fn with_hex_colors(mut self, background: &str, text: &str) -> Self {
        self.color =  String::from(background);
        self.color_text =  String::from(text);
        self
    }

    pub fn with_hex_color(mut self, text: &str) -> Self {

        let without_prefix = text.trim_start_matches("#");
        let r = u8::from_str_radix(without_prefix.get(..2).unwrap(), 16).unwrap();
        let g = u8::from_str_radix(without_prefix.get(2..4).unwrap(), 16).unwrap();
        let b = u8::from_str_radix(without_prefix.get(4..6).unwrap(), 16).unwrap();

        self.color_text =  format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, 255);
        self.color =  format!("#{:02x}{:02x}{:02x}{:02x}", r/3, g/3, b/3, 128);
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_adapter(mut self, adapter: &dyn TicketAdapter) -> Self {
        self.adapter = adapter.get_name();
        self
    }

    //hue 0-1,
    //saturation 0-1,
    //value 0-1
    pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {

        let v2 = (255.0 * v) as u8;

        if s == 0.0 {
            (v2, v2, v2)
        } else {
            let i = (h * 6.0) as u8;
            let f = ((h*6.0) as u8) as f32 - (i as f32);
            let p = (255.0 * (v * (1.0 - s))) as u8;
            let q = (255.0 * (v * (1.0 - s * f))) as u8;
            let t = (255.0 * (v * (1.0 - s * (1.0 - f)))) as u8; 

            match i%6 {
                0 => (v2, t, p),
                1 => (q, v2, p),
                2 => (p, v2, t),
                3 => (p, q, v2),
                4 => (t, p, v2),
                _ => (v2, p, q)
            }
        }
    }
}