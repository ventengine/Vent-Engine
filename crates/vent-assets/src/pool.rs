use std::collections::HashMap;

pub trait Asset {
    fn get_file_extensions() -> &'static str
    where
        Self: Sized;
}

struct AssetPool {
    assets: HashMap<String, Box<dyn Asset>>,
}

impl AssetPool {
    fn new() -> AssetPool {
        AssetPool {
            assets: HashMap::new(),
        }
    }

    fn add_asset<T: Asset + 'static>(&mut self, name: String, asset: T) {
        self.assets.insert(name, Box::new(asset));
    }

    fn remove_asset(&mut self, name: &str) -> Option<Box<dyn Asset>> {
        self.assets.remove(name)
    }

    fn get_asset(&self, name: &str) -> Option<&dyn Asset> {
        self.assets.get(name).map(|boxed| boxed.as_ref())
    }
}