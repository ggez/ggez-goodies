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
//! (In the same thread, at least.)
//!
//! What it does NOT do is allow you to free individual assets from
//! the cache.  This is on purpose.  If you want fine-grained manual
//! memory management you know where to get it.  This is more a memory
//! pool like thing where you allocate a bunch of objects, keep them
//! around for however long you need them (while the game is loaded,
//! while a particular scene is loaded, etc), and then free them all.
//!
//! If you want to make a stack of asset managers, where one
//! has access to the assets higher up in the stack...
//! Just build one and clone it.  All the Rc's in it will get
//! cloned along with it, providing exactly the behavior you
//! want except better.  :D
//!
//! Though whether or not asset handles from the old one will be valid
//! with the new one... hmmm.  That might not be a big problem since we
//! can just request new asset handles from the new cache and they'll already
//! be there, so that might be the way to go?

// TODO: This is not thread safe; should we offer one that it?
// TODO: Check out calx-resource:
// https://github.com/rsaarelm/calx/blob/master/calx-resource/src/lib.rs
// It has a) nifty macros to build these automatically,
// and b) an asset type that will serialize to a key, and then
// deserialize the key to that asset.  Very neat!  But also
// a bit labyrenthine.
// It DOES also make thread-safety work through thread_local!().

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::Debug;
use std::path::Path;
use std::rc::Rc;
use std::hash::Hash;
use ggez;
use ggez::{Context, GameError, GameResult};
use ggez::graphics;

pub trait Loadable<K, E> {
    fn load(_key: &K) -> Result<Self, E> where Self: Sized;
}

pub trait StateLoadable<K, E, S> {
    fn load_state(_key: &K, &mut S) -> Result<Self, E> where Self: Sized;
}

use std::marker::PhantomData;

/// An opaque asset handle that can be used for O(1) fetches
/// of assets.
// TODO: Add a UUID or something to this....
#[derive(Debug)]
pub struct Handle<T> {
    idx: usize,
    _phantom: PhantomData<*const T>,
}

impl<T> Copy for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Handle {
            idx: self.idx,
            _phantom: PhantomData,
        }
    }
}

// We COULD use a generic interning crate such as symtern or symbol-map to
// implement the Handle -> Asset map here.  It might be useful.
// But it wouldn't get us all the way because we'd still need to maintain
// the Key -> Handle association ourselves.
#[derive(Debug, Clone)]
pub struct AssetCache<K, V>
    where K: Hash + Eq + Clone + Debug
{
    handles: Vec<Rc<V>>,
    keys: HashMap<K, Handle<V>>,
    next_handle: usize,
}

impl<K, V> AssetCache<K, V>
    where K: Hash + Eq + Clone + Debug
{
    /// Creates a new `AssetCache` that loads assets
    /// when necessary with the given loader function.
    pub fn new() -> Self {
        AssetCache {
            handles: Vec::new(),
            keys: HashMap::new(),
            next_handle: 0,
        }
    }

    fn new_handle(&mut self) -> Handle<V> {
        let i = self.next_handle;
        self.next_handle += 1;
        Handle {
            idx: i,
            _phantom: PhantomData,
        }
    }

    // Inserts the given asset into the handles vec at the given
    // location, and inserts the key into the key->handle mapping.
    // Performs asserts that will panic if something
    // gets out of sync (which should be impossible).
    fn bind_handle(&mut self, key: K, h: Handle<V>, value: Rc<V>) {
        assert!(h.idx == self.handles.len());
        self.handles.push(value);

        assert!(!self.keys.contains_key(&key));
        self.keys.insert(key, h);
    }

    // Adds a new item to the cache, returns an Rc reference to it
    // and an Handle.
    fn add_item(&mut self, key: K, value: V) -> (Handle<V>, Rc<V>) {
        let handle = self.new_handle();
        let rc = Rc::new(value);
        self.bind_handle(key, handle, rc.clone());
        (handle, rc)
    }

    /// Retrieves an asset via its handle.
    /// This is always safe (and fast) because for a handle
    /// to be valid its object *must* exist in the cache.
    pub fn get(&self, handle: Handle<V>) -> Rc<V> {
        assert!(handle.idx < self.handles.len());
        self.handles[handle.idx].clone()
    }


    /// Not sure this is even right, but...
    pub fn get_mut<'a>(&'a mut self, handle: Handle<V>) -> Option<&'a mut V> {
        assert!(handle.idx < self.handles.len());
        use std::rc::Rc;
        Rc::get_mut(&mut self.handles[handle.idx])
    }


    // fn add_item_mut(&mut self, key: K, value: &mut V) -> (Handle, Rc<&mut V>) {
    //     let handle = self.new_handle();
    //     let entry = self.entry(key.clone());
    //     match entry {
    //         Entry::Vacant(e) => {
    //             let v = V::load(key)?;
    //             let v_rc = Rc::new(v);
    //             Ok(e.insert(v_rc));
    //             Ok(self.add_item(key.clone(), v));
    //         }
    //         Entry::Occupied(e) => Ok(e.into_mut()),
    //     }

    // }

    /// Gets the given asset, loading it if necessary.
    /// Returns an Rc to the value, plus an Handle
    /// which can be used to retrieve it quickly.
    // Oh my goodness getting the E type param to the
    // right place was amazingly difficult.
    pub fn get_key<E>(&mut self, key: &K) -> Result<(Handle<V>, Rc<V>), E>
        where V: Loadable<K, E>
    {
        if let Some(handle) = self.keys.get(key) {
            return Ok((*handle, self.get(*handle)));
        };

        let v = V::load(key)?;
        let res = self.add_item(key.clone(), v);
        Ok(res)
    }

    // pub fn get_mut<E, S>(&mut self, key: &K) -> Result<&mut Rc<V>, E>
    //     where V: Loadable<K, E>
    // {
    //     // entry.or_insert_with() isn't quite powerful
    //     // enough 'cause it doesn't propegate Results.  ;_;
    //     let entry = self.contents.entry(key.clone());
    //     match entry {
    //         Entry::Vacant(e) => {
    //             let v = V::load(key)?;
    //             let v_rc = Rc::new(v);
    //             Ok(e.insert(v_rc));
    //             Ok(self.add_item(key.clone(), v));
    //         }
    //         Entry::Occupied(e) => Ok(e.into_mut()),
    //     }
    // }

    /// Gets the given asset, loading it with a state object if necessary.
    pub fn get_key_state<E, S>(&mut self, key: &K, state: &mut S) -> Result<(Handle<V>, Rc<V>), E>
        where V: StateLoadable<K, E, S>
    {
        if let Some(handle) = self.keys.get(key) {
            return Ok((*handle, self.get(*handle)));
        };

        let v = V::load_state(key, state)?;
        let res = self.add_item(key.clone(), v);
        Ok(res)
    }

    // /// Removes all assets from the cache
    // /// and frees any excess memory it uses.
    // /// This is now unsafe because it introduces the possibility
    // /// of dangling handles!  We can have a stack of bindings, kinda,
    // /// to manage this.  (Or just create/delete new state objects.)
    // pub fn clear(&mut self) {
    //     let map = BTreeMap::new();
    //     self.contents = map;
    // }

    /// Returns true if the given asset is loaded.
    pub fn loaded(&self, key: &K) -> bool {
        self.keys.contains_key(key)
    }

    /// Takes a slice containing a list of keys,
    /// and loads all the keys so that their objects
    /// are immediately accessible.
    pub fn preload<E>(&mut self, keys: &[K])
        where V: Loadable<K, E>
    {
        for k in keys {
            let _ = self.get_key(k);
        }
    }

    /// Preloads objects that require a state to load.
    pub fn preload_state<E, S>(&mut self, keys: &[K], state: &mut S)
        where V: StateLoadable<K, E, S>
    {
        for k in keys {
            let _ = self.get_key_state(k, state);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    impl<'a> Loadable<&'a str, ()> for String {
        fn load(key: &&str) -> Result<String, ()> {
            Ok(key.to_string())
        }
    }

    impl<'a> StateLoadable<&'a str, (), i32> for String {
        fn load_state(key: &&str, state: &mut i32) -> Result<String, ()> {
            *state += 1;
            Ok(key.to_string())
        }
    }

    #[test]
    fn test_assetcache() {
        let mut a = AssetCache::<&str, String>::new();
        {
            assert!(!a.loaded(&"foo"));
            let (handle, s1) = a.get_key(&"foo").unwrap();
            assert!(a.loaded(&"foo"));
            assert_eq!(*s1, "foo");
            let gotten_with_handle = a.get(handle);
            assert_eq!(*s1, *gotten_with_handle);
        }
    }

    #[test]
    fn test_stateful_assetcache() {
        let mut a = AssetCache::<&str, String>::new();
        {
            let s = &mut 10;
            assert!(!a.loaded(&"foo"));
            let (handle, s1) = a.get_key_state(&"foo", s).unwrap();
            assert_eq!(*s1, "foo");
            assert_eq!(*s, 11);
            assert!(a.loaded(&"foo"));

            let gotten_with_handle = a.get(handle);
            assert_eq!(*s1, *gotten_with_handle);
            assert_eq!(*s, 11);

        }

    }

    #[test]
    fn test_mut_assetcache() {
        let mut a = AssetCache::<&str, String>::new();
        assert!(!a.loaded(&"foo"));
        let (h, _) = a.get_key(&"foo").unwrap();

        {
            let mut_value = a.get_mut(h.clone());
            assert!(mut_value.is_some());
            let m = mut_value.unwrap();
            assert_eq!("foo", *m);
            *m += "Foobaz";
            assert_eq!("fooFoobaz", *m);
        }
        // Now get it again and ensure it's mutated.
        {
            let mut_value = a.get_mut(h.clone());
            assert!(mut_value.is_some());
            let m = mut_value.unwrap();
            assert_eq!("fooFoobaz", *m);
        }

    }
}
