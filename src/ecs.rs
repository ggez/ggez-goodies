use std::fmt::Debug;

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

    fn register<T>(&mut self)
        where T: Default + 'static
    {
        self.components.insert(T::default());
    }

    fn run<F, T, Eoutput, Einput>(&mut self, inputs: Vec<Einput>, f: F) -> Vec<Eoutput>
        where F: Fn((&T, &Einput)) -> Eoutput + Sync,
              T: Debug + Send + Sync + 'static,
              Eoutput: Send + Sync,
              Einput: Send + Sync
    {
        if let Some(resource) = self.components.get::<VecResource<T>>() {
            let d: &[T] = &resource.data;
            let mut v = Vec::new();
            d.par_iter().zip(&inputs).map(f).collect_into(&mut v);
            v
        } else {
            panic!("Tried to run a system on an unknown component type");
        }
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
        let entity_count = 10;
        let mut w = World::new();
        for i in 0..entity_count {
            w.create_entity();
        }
        println!("Bar");
        let mut inputs = Vec::new();
        inputs.resize(entity_count, 1u32);
        let results = w.run(inputs, |(x1, x2): (&u32, &u32)| {
            println!("Xs are is {:?} {:?}", x1, x1);
            x1 + x2
        });
        println!("Results are {:?}", results);
        assert!(false);
    }
}
