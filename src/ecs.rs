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

#[derive(Clone, Debug)]
pub struct Entity(u32);

pub type InputChannel<T> = Vec<T>;
pub type OutputChannel<T> = mpsc::Sender<T>;

pub struct EventSender<E> {
    channels: Vec<RwLock<Vec<E>>>
}

// We never clone the channel senders, so I'm not sure if this is
// really better than just having a Vec<RwLock<E>>, but...
impl<E> EventSender<E> where E: Send + Sync {
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
        
        self.channels.par_iter()
            .for_each(|q| {
                // Not sure if this is actually an optimization but it might be?
                // Depends on how expensive getting a write lock is.
                // ...Probably not more expensive than getting a read lock, tbh.
                let to_clear = {
                    let q_read = &mut q.try_read().unwrap();
                    q_read.len() > 0
                };
                if to_clear {
                    let q_write = &mut q.try_write().expect("Tried to clear event queues while in use; should never happen!");
                    q_write.clear();
                }
            });
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

    pub fn register<T>(&mut self)
        where T: Default + 'static
    {
        self.current_components.insert(T::default());
        self.next_components.insert(T::default());
    }

    pub fn run1<F, C>(&mut self, f: F)
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

    pub fn run2<F, C1, C2>(&mut self, f: F)
        where F: Fn(&C1, &C2, &[E], &EventSender<E>) -> (C1, C2) + Sync,
              C1: Debug + Send + Sync + 'static,
              C2: Debug + Send + Sync + 'static
    {
        let current1 = self.current_components.get::<VecResource<C1>>().expect("Tried to run a system on an unknown component type");
        let current2 = self.current_components.get::<VecResource<C2>>().expect("Tried to run a system on an unknown component type");            
        let c1: &[C1] = &current1.data;
        let c2: &[C2] = &current2.data;
        
        let next_events = &self.next_events;
        // BUGGO: Aieee, my perfect non-allocating system is now poisoned!
        let mut next_hax: Vec<(C1, C2)> = Vec::with_capacity(c1.len());
        c1.par_iter()
            .zip(c2)
            .zip(&self.current_events.channels)
            .map(|((comp1, comp2), e)| {
                let event_queue = e.read().expect("Aiee event queue is poisoned in World::run(); did a system crash?");
                f(comp1, comp2, &event_queue[..], next_events)
            })
            .collect_into(&mut next_hax);
        //.enumerate()
        // This doesn't seem to work 'cause it gets pesky about the closure altering self,
        // for some reason.  Hmm.
        //.for_each(|(i, (comp1, comp2))| {
            //    next1.data[i] = comp1;
            //});
        //.collect_into(v);
        let (r1, r2): (Vec<C1>, Vec<C2>) = next_hax.into_iter().unzip();
        {
            let next1 = self.next_components.get_mut::<VecResource<C1>>().expect("current_components exists but next_components does not, this should never happen!");
            let n1 = &mut next1.data;
            *n1 = r1;
        }
        {
            let next2 = self.next_components.get_mut::<VecResource<C2>>().expect("current_components exists but next_components does not, this should never happen!");
            let n2 = &mut next2.data;
            *n2 = r2;
        }
    }


    // This function finalizes the end of the frame.
    // Delivers events, flips the current and next components, etc.
    pub fn finish(&mut self) {
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

    pub fn create_entity<C1, C2>(&mut self, component1: C1, component2: C2) -> Entity
        where C1: Debug + Send + Sync + Clone + 'static,
              C2: Debug + Send + Sync + Clone + 'static,
    {
        let e = self.next_entity();
        self.entities.push(e.clone());
        self.current_events.channels.push(RwLock::new(Vec::new()));
        self.next_events.channels.push(RwLock::new(Vec::new()));

        {
            let nc: &mut VecResource<C1> = self.next_components.entry().or_insert_with(VecResource::new);
            nc.data.push(component1.clone());
            let components: &mut VecResource<C1> = self.current_components.entry().or_insert_with(VecResource::new);
            components.data.push(component1);
        }
        {
            let nc: &mut VecResource<C2> = self.next_components.entry().or_insert_with(VecResource::new);
            nc.data.push(component2.clone());
            let components: &mut VecResource<C2> = self.current_components.entry().or_insert_with(VecResource::new);
            components.data.push(component2);
        }

        
        e
    }

    /*
Uninitialized components makes this tricky,
as does making sure we register all component types
before adding them.
Not impossible, just takes a little finesse.
How does specs do it?
    fn add_component<C>(&mut self, entity: Entity, component: C)
        where C: Debug + Send + Sync + Clone + 'static {
        let nc: &mut VecResource<T> = self.next_components.entry().or_insert_with(VecResource::new);
        nc.data.push(component.clone());
        let components: &mut VecResource<T> = self.current_components.entry().or_insert_with(VecResource::new);
        components.data.push(component);
    }
*/


    pub fn send_to_entity(&mut self, entity: Entity, event: E) {
        self.next_events.send_to_entity(entity, event);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rand;

    #[test]
    fn test_world_thingy() {
        let entity_count = 100;
        let message_count = 1000;
        let loops = 100;
        let mut w = World::new();
        for i in 0..entity_count {
            w.create_entity(i as usize, ());
        }
        for i in 0..message_count {
            let dest = rand::random::<u32>() % (entity_count as u32);
            w.send_to_entity(Entity(dest), 0);
        }
        for _ in 0..loops {
            // Call finish to make the event routing happen.
            w.finish();
            w.run1(|comp: &usize, events: &[usize], writer: &EventSender<usize>| {
                // println!("Component: {} Event: {:?}", comp, events);
                // Just send any event you get to the next entity index;
                // the event number is how many times it's been sent.
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
