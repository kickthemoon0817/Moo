#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug, Clone)]
pub struct UiButton {
    pub label: String,
    pub rect: Rect,
    pub background: Color,
}

#[derive(Debug, Clone)]
pub enum UiElement {
    Button(UiButton),
}

impl UiElement {
    pub fn button(label: impl Into<String>, rect: Rect, background: Color) -> Self {
        UiElement::Button(UiButton {
            label: label.into(),
            rect,
            background,
        })
    }
}
