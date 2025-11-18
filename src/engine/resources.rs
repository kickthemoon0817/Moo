use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct ResourceManager {
    textures: HashMap<String, String>,
}

impl ResourceManager {
    pub fn register_texture(&mut self, key: impl Into<String>) {
        let key = key.into();
        tracing::debug!(%key, "registering texture placeholder");
        self.textures.entry(key).or_insert_with(String::new);
    }

    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }
}
