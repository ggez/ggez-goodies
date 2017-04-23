use std::fmt::Debug;

use rayon::prelude::*;
use anymap;

#[derive(Clone, Default, Debug)]
pub struct Component {
    data: i32,
}


// #[derive(Clone, Default)]
// pub struct VecResource<T> {
//     data: Vec<T>,
// }


#[derive(Clone, Default, Debug)]
pub struct VecResource<T> where T: Debug {
    data: Vec<T>,
}

impl<T> VecResource<T> where T: Debug {
    pub fn new() -> Self {
        VecResource {
            data: Vec::new(),
        }
    }
}


pub struct Entity((u32, u32));

pub struct World {
    entities: Vec<Entity>,
    components: anymap::AnyMap,
}

impl World {
    pub fn new() -> Self {
        World {
            entities: Vec::new(),
            components: anymap::AnyMap::new(),
        }
    }

    fn register<T>(&mut self) where T: Default + 'static {
        self.components.insert(T::default());
    }

    fn run<F, T>(&mut self, f: F) where F: Fn(&T) + Sync, T: Debug + Send {
        println!("Foo");
        // if let Some(resource) = self.components.get::<VecResource<T>>() {
        //     for x in  resource.data
        //         .iter() {
        //              println!("{:?}", x);
        //         }

        //     let d: &[T] = &resource.data;
        //     // d.par_iter();
        // } else {
        //     panic!("Tried to run a system on an unknown component type");
        // }

        let v: &Vec<u32> = &Vec::new();
        v.par_iter();
    }

    fn create_entity(&mut self) {
        self.entities.push(Entity((0, 0)));
        let vr: &mut VecResource<u32> = self.components.entry().or_insert_with(VecResource::new);
        vr.data.push(self.entities.len() as u32);

    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_world_thingy() {
        let mut w = World::new();
        for i in 0..10 {
            w.create_entity();
        }
        println!("Bar");
        w.run(|x:&u32| ());
        assert!(false);
    }
}