use std::fmt::Debug;
use std::mem;
use std::sync::mpsc;
use std::sync::RwLock;

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

pub type InputChannel<T> = Vec<T>;
pub type OutputChannel<T> = mpsc::Sender<T>;

pub struct EventSender<E> {
    channels: Vec<RwLock<Vec<E>>>
}

// We never clone the channel senders, so I'm not sure if this is
// really better than just having a Vec<RwLock<E>>, but...
impl<E> EventSender<E> where E: Send {
    fn new() -> Self {
        EventSender {
            channels: Vec::new(),
        }
    }
    
    pub fn send_to_entity(&self, entity: Entity, event: E) {
        let channel = self.channels.get(entity.0 as usize).expect("Sent message to non-existent entity!");
        let channel_write = &mut channel.write().expect("Attempted to write on poisoned RwLock, aiee!");
        channel_write.push(event);
    }

    pub fn clear(&mut self) {
        // Might as well parallelize?
        // Overhead might not make it worth it.
        // Try it later, see if it matters.
        /*
        self.channels.par_iter_mut()
            .for_each(|q| {
                let q_write = q.try_write().expect("Tried to clear event queues while in use; should never happen!");
                *q_write.clear()
            });
         */
        for q in self.channels.iter_mut() {
            let q_write = &mut q.try_write().expect("Tried to clear event queues while in use; should never happen!");
            q_write.clear()
        }
    }
}

// I feel a little ghetto making one global event type for everything,
// but we'll roll with it for now.
pub struct World<E> where E: Send {
    entities: Vec<Entity>,
    current_components: anymap::AnyMap,
    next_components: anymap::AnyMap,
    current_events: EventSender<E>,
    next_events: EventSender<E>,
}


impl<E> World<E> where E: Send + Sync {
    pub fn new() -> Self {
        World {
            entities: Vec::new(),
            current_components: anymap::AnyMap::new(),
            next_components: anymap::AnyMap::new(),
            current_events: EventSender::new(),
            next_events: EventSender::new(),
        }
    }

    fn register<T>(&mut self)
        where T: Default + 'static
    {
        self.current_components.insert(T::default());
        self.next_components.insert(T::default());
    }

    fn run<F, C>(&mut self, f: F)
        where F: Fn(&C, &[E], &EventSender<E>) -> C + Sync,
              C: Debug + Send + Sync + 'static
    {
        if let Some(resource) = self.current_components.get::<VecResource<C>>() {
            if let Some(next_components) = self.next_components.get_mut::<VecResource<C>>() {
                let d: &[C] = &resource.data;
                let v = &mut next_components.data;
                let next_events = &self.next_events;
                d.par_iter()
                    .zip(&self.current_events.channels)
                    .map(|(c, e)| {
                        let event_queue = e.read().expect("Aiee event queue is poisoned in World::run()");
                        f(c, &event_queue[..], next_events)
                    })
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

        // empty out old events, deliver new ones.
        // Making the iterator work easily takes a little
        // bit of explicit type-mongling
        {
            let e1 = &mut self.current_events;
            let e2 = &mut self.next_events;
            mem::swap(e1, e2);
        }
        self.next_events.clear();
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

        self.current_events.channels.push(RwLock::new(Vec::new()));
        self.next_events.channels.push(RwLock::new(Vec::new()));
    }

    fn send_to_entity(&mut self, entity: Entity, event: E) {
        self.next_events.send_to_entity(entity, event);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_world_thingy() {
        let entity_count = 10;
        let loops = 10;
        let mut w = World::new();
        for i in 0..entity_count {
            w.create_entity(i as usize);
        }
        w.send_to_entity(Entity(3), 0);
        for _ in 0..loops {
            // Call finish to make the event routing happen.
            w.finish();
            let results = w.run(|comp: &usize, events: &[usize], writer: &EventSender<usize>| {
                println!("Component: {} Event: {:?}", comp, events);
                for e in events {
                    writer.send_to_entity(Entity(((*comp+1) % entity_count) as u32), *e + 1);
                }
                *comp
            });
            //println!("Entit are {:?}", results);
        }
        let mut desired_results: Vec<u32> = Vec::new();
        desired_results.extend(&[2, 3, 4, 5, 6, 7, 8, 9, 10, 11][..]);
        assert!(false);
    }
}
