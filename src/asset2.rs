//! An experimental asset loader loosely based off
//! of a ggez-ified version of Amethyst's asset loader:
//!
//! https://docs.rs/amethyst/0.4.3/amethyst/asset_manager/index.html
//!
//! The main difference from Amethyst is it doesn't store things in specs;
//! the main difference from the existing asset loader is its storage of
//! assets in anymap's.

use std::collections::HashMap;
use std::any::{Any, TypeId};
use ggez;

pub type AssetId = usize;

pub struct AssetCache {
    loaders: HashMap<TypeId, Box<AssetLoader<Box<Any>>>>,
    asset_ids: HashMap<String, AssetId>,
    assets: Vec<Box<Any>>,
}

/// Describes an abstract asset loader type.
///
/// This is what Amethyst uses but is weirdly not-what-we-want for
/// this application, so idk.
//pub trait AssetLoader<A, E> {
//    fn from_data(assets: &mut AssetCache, data: Self) -> Result<A, E>;
//}

/// This isn't what we want either though, apparently.  So!
pub type AssetLoader<A> = Fn(&mut ggez::Context, &str) -> ggez::GameResult<A>;

impl AssetCache {
    pub fn new() -> Self {
        Self {
            loaders: HashMap::new(),
            asset_ids: HashMap::new(),
            assets: Vec::new(),
        }
    }
    
    pub fn add_loader<T: Any>(&mut self, loader: Box<AssetLoader<T>>) {
        //let loader = Box::new(loader);
        self.loaders.insert(TypeId::of::<T>(), loader as Box<AssetLoader<Box<Any>>>);
    }

    /// Load an asset from data
    pub fn load_asset_from_data<A>(&mut self,
                                   ctx: &mut ggez::Context,
                                   name: &str)
                                   -> ggez::GameResult<AssetId>
        where A: Any
    {
        //let asset = AssetLoader::<A, E>::from_data(self, data)?;
        let loader = self.loaders.get(&TypeId::of::<A>()).unwrap();
        let asset = loader(ctx, name)?;
        let id = self.add_asset(name, asset);
        Ok(id)
    }

    pub fn id_from_name(&self, name: &str) -> Option<AssetId> {
        self.asset_ids.get(name).map(|id| *id)
    }

    pub fn get<T>(&self, id: AssetId) -> Option<&T>
        where T: 'static
    {
        self.assets.get(id)
            .map(|itm| &**itm)
            .and_then(|itm| itm.downcast_ref::<T>())
    }

    fn add_asset<A: Any>(&mut self, name: &str, asset: A) -> AssetId {
        self.assets.push(Box::new(asset));
        let id = self.assets.len();
        self.asset_ids
            .entry(name.into())
            .or_insert(id);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Hash, PartialEq)]
    struct DummyImage(usize);
    #[derive(Clone, Debug, Hash, PartialEq)]
    struct DummyImageData;
    impl AssetLoader<DummyImage, ()> for DummyImageData {
        fn from_data(assets: &mut AssetCache, data: Self) -> Result<DummyImage, ()> {
            Ok(DummyImage(1))
        }
    }

    #[test]
    fn test_loading() {
        let mut cache = AssetCache::new();
        cache.add_loader(DummyImageData);
        let id = cache.load_asset_from_data("foo", DummyImageData).unwrap();
        //let itm = cache.get::<DummyImageData>(id).unwrap();
        //assert_eq!(itm, &DummyImage(1));
    }
}
