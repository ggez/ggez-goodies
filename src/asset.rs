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
// TODO: Check out calx-resource:
// https://github.com/rsaarelm/calx/blob/master/calx-resource/src/lib.rs
// It has a) nifty macros to build these automatically,
// and b) an asset type that will serialize to a key, and then
// deserialize the key to that asset.  Very neat!  But also
// a bit labyrenthine.
// It DOES also make thread-safety work through thread_local!().


use std::collections::BTreeMap;
use std::error::Error;
use std::rc::Rc;

pub trait Loadable<K,E> {
    fn load(_key: &K) -> Result<Self, E> where Self: Sized;
}

pub struct AssetCache<K, V>
    where K: Ord + Clone
{
    contents: BTreeMap<K, Rc<V>>,
}

impl<K, V> AssetCache<K, V> where K: Ord + Clone
{
    /// Creates a new `AssetCache` that loads assets
    /// when necessary with the given loader function.
    pub fn new() -> Self {
        let map = BTreeMap::new();
        AssetCache { contents: map }
    }

    /// Gets the given asset, loading it if necessary.
    // Oh my goodness getting the E type param to the
    // right place was amazingly difficult.
    pub fn get<E>(&mut self, key: &K) -> Result<Rc<V>, E>
        where V: Loadable<K, E>
    {
        if let Some(v) = self.contents.get(key) {
            return Ok(v.clone());
        };

        let v = try!(V::load(key));
        let v_rc = Rc::new(v);
        self.contents.insert(key.clone(), v_rc.clone());
        Ok(v_rc)
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
    pub fn preload<E>(&mut self, keys: &[K])
        where V: Loadable<K, E>
    {
        for k in keys {
            let _ = self.get(k);
        }
    }
}


pub trait StateLoadable<K,E,S> {
    fn load(_key: &K, &mut S) -> Result<Self, E> where Self: Sized;
}

/// An AssetCache that owns some State
/// object, which then gets passed to and
/// updated by the Load function.
pub struct StatefulAssetCache<K, V, S>
    where K: Ord + Clone
{
    contents: BTreeMap<K, Rc<V>>,
    state: S,
}

impl<K, V, S> StatefulAssetCache<K, V, S> where K: Ord + Clone
{
    /// Creates a new `AssetCache` that loads assets
    /// when necessary with the given loader function.
    pub fn new(state: S) -> Self {
        let map = BTreeMap::new();
        StatefulAssetCache {
            contents: map,
            state: state,
        }
    }

    /// Gets the given asset, loading it if necessary.
    pub fn get<E>(&mut self, key: &K) -> Result<Rc<V>, E>
        where V: StateLoadable<K, E, S>
    {
        if let Some(v) = self.contents.get(key) {
            return Ok(v.clone());
        };

        let v = try!(V::load(key, &mut self.state));
        let v_rc = Rc::new(v);
        self.contents.insert(key.clone(), v_rc.clone());
        Ok(v_rc)
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
    pub fn preload<E>(&mut self, keys: &[K])
        where V: StateLoadable<K, E, S>
    {
        for k in keys {
            let _ = self.get(k);
        }
    }
}


mod tests {
    use super::*;

    impl<'a> Loadable<&'a str, ()> for String {
        fn load(key: &&str) -> Result<String, ()> {
            Ok(key.to_string())
        }
    }

    // It would be nice to get rid of the double references here somehow,
    // but then AssetCache ends up with a type of <str, String>
    // and `str` is not sized so it always has to involve a reference.
    #[test]
    fn test_assetcache() {
        let mut a = AssetCache::<&str, String>::new();
        assert!(!a.loaded(&"foo"));
        let s1 = a.get(&"foo").unwrap();
        assert_eq!(*s1, "foo");
        assert!(a.loaded(&"foo"));
    }


    impl<'a> StateLoadable<&'a str, (), i32> for String {
        fn load(key: &&str, state: &mut i32) -> Result<String, ()> {
            *state += 1;
            Ok(key.to_string())
        }
    }

    #[test]
    fn test_stateful_assetcache() {
        let mut a = StatefulAssetCache::<&str, String, i32>::new(0);
        assert_eq!(a.state, 0);
        assert!(!a.loaded(&"foo"));
        let s1 = a.get(&"foo").unwrap();
        assert_eq!(*s1, "foo");
        assert_eq!(a.state, 1);
        assert!(a.loaded(&"foo"));
    }

}
