use std::collections::HashMap;

pub struct TextureManager {
    dict: HashMap<&str, &str>,
}

impl TextureManager {

    pub fn new() -> TextureManager {
        let dict: HashMap<&str, &str> = HashMap::new();
        TextureManager {
            dict,
        }
    }

    pub fn load_target(&mut self) {

    }
}
