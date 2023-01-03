use egui::{Response, RichText};

use crate::UITheme;

pub struct UIHelper {}

impl UIHelper {

    pub fn extend_tooltip(response: Response, ui_theme: &UITheme, text: &str) -> Response {
        response.on_hover_text_at_pointer(UIHelper::sized_text(ui_theme, 1.0, text))
    }

    pub fn sized_text(ui_theme: &UITheme, factor: f32, text: &str) -> RichText {
        RichText::new(text).size(ui_theme.font_size as f32 * factor)
    }

}