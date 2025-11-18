use crate::ui::{Color, Rect, UiElement};

#[derive(Debug, Clone)]
pub struct StartWindow {
    pub title: &'static str,
    pub subtitle: &'static str,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    buttons: Vec<&'static str>,
}

impl Default for StartWindow {
    fn default() -> Self {
        Self {
            title: "Sandbox Prototype",
            subtitle: "Ready to explore?",
            width: 1024,
            height: 640,
            resizable: true,
            buttons: vec!["Start", "Options", "Credits", "Quit"],
        }
    }
}

impl StartWindow {
    pub fn button_elements(&self) -> Vec<UiElement> {
        let button_width = 360.0;
        let button_height = 64.0;
        let vertical_spacing = 18.0;
        let center_x = (self.width as f32 - button_width) / 2.0;
        let start_y = (self.height as f32 * 0.45) - button_height / 2.0;

        self.buttons
            .iter()
            .enumerate()
            .map(|(idx, label)| {
                let y = start_y + idx as f32 * (button_height + vertical_spacing);
                let color = if idx == 0 {
                    Color::rgba(0.62, 0.36, 0.94, 0.92)
                } else {
                    Color::rgba(0.28, 0.31, 0.51, 0.9)
                };
                UiElement::button(
                    format!("{label}"),
                    Rect {
                        x: center_x,
                        y,
                        width: button_width,
                        height: button_height,
                    },
                    color,
                )
            })
            .collect()
    }
}
