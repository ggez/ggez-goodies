use std::fmt::Debug;
use std::sync::mpsc;

use rayon::prelude::*;
use rayon;
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
pub struct VecResource<T>
    where T: Debug
{
    data: Vec<T>,
}

impl<T> VecResource<T>
    where T: Debug
{
    pub fn new() -> Self {
        VecResource { data: Vec::new() }
    }
}


pub struct Entity(u32);

// I feel a little ghetto making one global event type for everything,
// but we'll roll with it for now.
pub struct World<E> {
    entities: Vec<Entity>,
    components: anymap::AnyMap,
    inputs: Vec<InputChannel<E>>,
    outputs: Vec<OutputChannel<E>>
}

pub type InputChannel<T> = mpsc::Receiver<T>;
pub type OutputChannel<T> = mpsc::Sender<T>;

impl<E> World<E> where E: Send + Sync {
    pub fn new() -> Self {
        World {
            entities: Vec::new(),
            components: anymap::AnyMap::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    fn register<T>(&mut self)
        where T: Default + 'static
    {
        self.components.insert(T::default());
    }

    fn run<F, C>(&mut self, inputs: Vec<E>, f: F) -> Vec<C>
        where F: Fn((&C, &E)) -> C + Sync,
              C: Debug + Send + Sync + 'static
    {
        if let Some(resource) = self.components.get::<VecResource<C>>() {
            let d: &[C] = &resource.data;
            let mut v = Vec::new();
            d.par_iter().zip(&inputs).map(f).collect_into(&mut v);
            v
        } else {
            panic!("Tried to run a system on an unknown component type");
        }
    }

    fn next_entity(&self) -> Entity {
        use std::u32;
        assert!(self.entities.len() < u32::MAX as usize);
        Entity(self.entities.len() as u32)
    }

    fn create_entity<T>(&mut self, component: T) where T: Debug + Send + Sync + 'static {
        let e = self.next_entity();
        self.entities.push(e);
        let vr: &mut VecResource<T> = self.components.entry().or_insert_with(VecResource::new);
        vr.data.push(component);

        let (tx, rx) = mpsc::channel();
        self.inputs.push(rx);
        self.outputs.push(tx);
    }

    // fn send_to_entity(&self, entity: Entity, event: E) {
    //     self.
    // }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_world_thingy() {
        let entity_count = 10;
        let mut w = World::new();
        for i in 0..entity_count {
            w.create_entity(i*i);
        }
        let mut inputs = Vec::new();
        inputs.resize(entity_count, 1usize);
        // Event input just being a vector is tricksy because it may not
        // be the same length as the entities vector.  It needs to be a particular
        // per-entity mapping of one kind or another.
        // Making it efficient so we don't need to always re-allocate a new vector
        // for returning new events will be a little wacky too.
        let results = w.run(inputs, |(comp, event): (&u32, &usize)| {
            println!("Component: {} Event: {}", comp, event);
            comp + (*event as u32)
        });
        println!("Results are {:?}", results);
        let mut desired_results: Vec<u32> = Vec::new();
        desired_results.extend(&[2, 3, 4, 5, 6, 7, 8, 9, 10, 11][..]);
        assert_eq!(results, desired_results);
    }
}
