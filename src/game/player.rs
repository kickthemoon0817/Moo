#[derive(Debug, Default)]
pub struct Player {
    pub name: String,
}

impl Player {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}
