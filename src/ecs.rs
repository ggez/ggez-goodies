use std::fmt::Debug;
use std::mem;
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
    current_components: anymap::AnyMap,
    next_components: anymap::AnyMap,
    current_events: Vec<InputChannel<E>>,
    next_events: Vec<OutputChannel<E>>
}

pub type InputChannel<T> = Vec<T>;
pub type OutputChannel<T> = Vec<T>;

impl<E> World<E> where E: Send + Sync {
    pub fn new() -> Self {
        World {
            entities: Vec::new(),
            // We 
            current_components: anymap::AnyMap::new(),
            next_components: anymap::AnyMap::new(),
            current_events: Vec::new(),
            next_events: Vec::new(),
        }
    }

    fn register<T>(&mut self)
        where T: Default + 'static
    {
        self.current_components.insert(T::default());
        self.next_components.insert(T::default());
    }

    fn run<F, C>(&mut self, inputs: Vec<E>, f: F)
        where F: Fn(&C, &[E]) -> C + Sync,
              C: Debug + Send + Sync + 'static
    {
        if let Some(resource) = self.current_components.get::<VecResource<C>>() {
            if let Some(next_components) = self.next_components.get_mut::<VecResource<C>>() {
                let d: &[C] = &resource.data;
                let v = &mut next_components.data;
                let current_events: &[Vec<E>] = &self.current_events;
                d.par_iter()
                    .zip(current_events)
                    .map(|(c, e)| f(c, &e[..]))
                    .collect_into(v);
            } else {
                panic!("current_components exists but next_components does not, this should never happen!")
            }
        } else {
            panic!("Tried to run a system on an unknown component type");
        }
    }

    // This function finalizes the end of the frame.
    // Delivers events, flips the current and next components, etc.
    fn finish(&mut self) {
        // Switch components
        {
            let m1 = &mut self.current_components;
            let m2 = &mut self.next_components;
            mem::swap(m1, m2);
        }

        // Deliver events, empty out old ones.
        for event_queue in &mut self.current_events {
            event_queue.drain(..);
        }
        {
            let m1 = &mut self.current_events;
            let m2 = &mut self.next_events;
            mem::swap(m1, m2);
        }
    }

    fn next_entity(&self) -> Entity {
        use std::u32;
        assert!(self.entities.len() < u32::MAX as usize);
        Entity(self.entities.len() as u32)
    }

    fn create_entity<T>(&mut self, component: T) where T: Debug + Send + Sync + Clone + 'static {
        let e = self.next_entity();
        self.entities.push(e);
        
        let nc: &mut VecResource<T> = self.next_components.entry().or_insert_with(VecResource::new);
        nc.data.push(component.clone());
        let components: &mut VecResource<T> = self.current_components.entry().or_insert_with(VecResource::new);
        components.data.push(component);

        self.current_events.push(Vec::new());
        self.next_events.push(Vec::new());
    }

    fn send_to_entity(&mut self, entity: Entity, event: E) {
        self.next_events[entity.0 as usize].push(event);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_world_thingy() {
        let entity_count = 10;
        let mut w = World::new();
        for i in 0..entity_count {
            w.create_entity((i*i) as u32);
        }
        let mut inputs = Vec::new();
        inputs.resize(entity_count, 1usize);
        // Event input just being a vector is tricksy because it may not
        // be the same length as the entities vector.  It needs to be a particular
        // per-entity mapping of one kind or another.
        // Making it efficient so we don't need to always re-allocate a new vector
        // for returning new events will be a little wacky too.
        let results = w.run(inputs, |comp: &u32, events: &[usize]| {
            println!("Component: {} Event: {:?}", comp, events);
            comp + (*events.get(0).unwrap_or(&1) as u32)
        });
        println!("Results are {:?}", results);
        let mut desired_results: Vec<u32> = Vec::new();
        desired_results.extend(&[2, 3, 4, 5, 6, 7, 8, 9, 10, 11][..]);
        assert!(false);
    }
}
