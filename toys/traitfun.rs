use std::collections::HashMap;
use std::hash::Hash;

pub trait PgCopyIn
where
    Self: 'static,
{
    fn copy_in<'a, I>(items: I)
    where
        I: IntoIterator<Item = &'a Self>;
}

trait Batcher {
    type Item;
    fn flush(&mut self);
    fn add(&mut self, item: Self::Item);
    fn new() -> Self;
}

trait HasKey {
    type K: Eq + Hash;
    fn key(&self) -> Self::K;
}

/////////////////////////// MapBatcher
struct MapBatcher<V>
where
    V: HasKey,
{
    data: HashMap<V::K, V>,
}

impl<T: PgCopyIn + HasKey> Batcher for MapBatcher<T> {
    type Item = T;
    fn flush(&mut self) {
        T::copy_in(self.data.values());
        self.data.clear();
    }
    fn new() -> Self {
        return MapBatcher {
            data: HashMap::new(),
        };
    }
    fn add(&mut self, item: T) {
        self.data.insert(item.key(), item);
    }
}

/////////////////////// VecBatcher
struct VecBatcher<V> {
    data: Vec<V>,
}

impl<T: PgCopyIn> Batcher for VecBatcher<T> {
    type Item = T;
    fn flush(&mut self) {
        T::copy_in(&self.data);
        self.data.clear();
    }
    fn add(&mut self, item: Self::Item) {
        self.data.push(item);
    }
    fn new() -> Self {
        return VecBatcher { data: Vec::new() };
    }
}

/////////////////// Cake

#[derive(Debug, Clone)]
struct Cake {
    name: String,
    topping: String,
}

#[derive(Eq, PartialEq, Hash)]
struct CakeKey {
    kname: String,
}

impl HasKey for Cake {
    type K = CakeKey;
    fn key(&self) -> CakeKey {
        return CakeKey {
            kname: self.name.clone(),
        };
    }
}

impl PgCopyIn for Cake {
    fn copy_in<'a, I>(items: I)
    where
        I: IntoIterator<Item = &'a Self>,
    {
        for item in items {
            println!("{:?}", item)
        }
    }
}

fn main() {
    let c1 = Cake {
        name: String::from("yay"),
        topping: String::from("bannana"),
    };
    let c2 = Cake {
        name: String::from("yay"),
        topping: String::from("apple"),
    };
    let mut b = MapBatcher::<Cake>::new();
    b.add(c1.clone());
    b.add(c2.clone());
    b.flush();

    println!("now with VecBatcher");
    let mut vb = VecBatcher::<Cake>::new();
    vb.add(c1);
    vb.add(c2);
    vb.flush();
}
