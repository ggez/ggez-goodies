//! This module implements a simple asset loader and cache.
//!
//! While ggez offers functions to load raw game assets such as
//! music files, images, etc, it provides no facilities for managing
//! them.  Ideally you want to load all the assets you need at once,
//! hold on to them for a time (the course of a level for instance),
//! and then potentially ditch the ones you don't need.
//! This module offers cache-like functionality to do exactly this.
//! 
//! It will return an `Rc` containing the item loaded, so multiple
//! items can safely access (read-only) instances of the same asset.

// TODO: This is not thread safe; should we offer one that it?


use std::collections::BTreeMap;
use std::rc::Rc;
use std::boxed::Box;

pub struct AssetCache<K,V>
    where K: Ord + Clone {
    contents: BTreeMap<K,Rc<V>>,
    loader: Box<FnMut(&K) -> V>,
}

impl<K,V> AssetCache<K,V>
    where K: Ord + Clone {
    /// Creates a new `AssetCache` that loads assets
    /// when necessary with the given loader function.
    pub fn new(loaderfunc: Box<FnMut(&K) -> V>) -> Self {
        let map = BTreeMap::new();
        AssetCache {
            contents: map,
            loader: loaderfunc,
        }
    }

    /// Gets the given asset, loading it if necessary.
    pub fn get(&mut self, key: &K) -> Rc<V> {
        let loader = &mut self.loader;
        let f = || {
            let item = loader(key);
            Rc::new(item)
        };
        let entry = self.contents.entry(key.clone());
        //let loader = self.loader;
        entry.or_insert_with(f).clone()
    }

    /// Removes all assets from the cache
    /// and frees any excess memory it uses.
    pub fn clear(&mut self) {
        let map = BTreeMap::new();
        self.contents = map;
    }

    /// Returns true if the given asset is loaded.
    pub fn loaded(&self, key: &K) -> bool {
        self.contents.contains_key(key)
    }

    /// Takes a slice containing a list of keys,
    /// and loads all the keys so that their objects
    /// are immediately accessible.
    pub fn preload(&mut self, keys: &[K]) {
        for k in keys {
            let _ = self.get(k);
        }
    }
}

pub struct StatefulAssetCache<K,V,S>
    where K: Ord + Clone {
    contents: BTreeMap<K,Rc<V>>,
    loader: fn(&K, &mut S) -> V,
    state: S,
}

impl<K,V,S> StatefulAssetCache<K,V,S>
    where K: Ord + Clone {
    /// Creates a new `AssetCache` that loads assets
    /// when necessary with the given loader function.
    pub fn new(loaderfunc: fn(&K, &mut S) -> V, state: S) -> Self {
        let map = BTreeMap::new();
        StatefulAssetCache {
            contents: map,
            loader: loaderfunc,
            state: state,
        }
    }

    /// Gets the given asset, loading it if necessary.
    pub fn get(&mut self, key: &K) -> Rc<V> {
        let loader = self.loader;
        let state = &mut self.state;
        let f = || {
            let item = loader(key, state);
            Rc::new(item)
        };
        let entry = self.contents.entry(key.clone());
        //let loader = self.loader;
        entry.or_insert_with(f).clone()
    }

    /// Removes all assets from the cache
    /// and frees any excess memory it uses.
    pub fn clear(&mut self) {
        let map = BTreeMap::new();
        self.contents = map;
    }

    /// Returns true if the given asset is loaded.
    pub fn loaded(&self, key: &K) -> bool {
        self.contents.contains_key(key)
    }

    /// Takes a slice containing a list of keys,
    /// and loads all the keys so that their objects
    /// are immediately accessible.
    pub fn preload(&mut self, keys: &[K]) {
        for k in keys {
            let _ = self.get(k);
        }
    }
}


mod tests {
    use super::*;

    // It would be nice to get rid of the double references here somehow,
    // but then AssetCache ends up with a type of <str, String>
    // and `str` is not sized so it always has to involve a reference.
    #[test]
    fn test_assetcache() {
        //let mut loaderState = 0;
        let loader = |s: &&str| {
            //loaderState += 1;
            match *s {
                "foo" => "FooBaz".to_owned(),
                "bar" => "BarBaz".to_owned(),
                _ => "Something else".to_owned(),
            }
        };

        let mut a = AssetCache::new(Box::new(loader));
        assert!(!a.loaded(&"foo"));
        let s1 = a.get(&"foo");
        assert_eq!(*s1, "FooBaz".to_owned());
        assert!(a.loaded(&"foo"));
    }

    #[test]
    fn test_stateful_assetcache() {
        fn loader(s: &&str, loaderinfo: &mut u32) -> String {
            *loaderinfo += 1;
            match *s {
                "foo" => "FooBaz".to_owned(),
                "bar" => "BarBaz".to_owned(),
                _ => "Something else".to_owned(),
            }
        }

        let mut a = StatefulAssetCache::new(loader, 0);
        assert!(!a.loaded(&"foo"));
        let s1 = a.get(&"foo");
        assert_eq!(*s1, "FooBaz".to_owned());
        assert!(a.loaded(&"foo"));
    }

}
