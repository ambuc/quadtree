use std::collections::{BTreeMap, HashMap};

use num::PrimInt;

use crate::entry::Entry;

pub trait Map<U, V>
where
    U: PrimInt + Default + 'static,
{
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get(&self, k: u64) -> Option<&Entry<U, V>>;
    fn get_mut(&mut self, k: u64) -> Option<&mut Entry<U, V>>;
    fn clear(&mut self);
    fn insert(&mut self, k: u64, v: Entry<U, V>) -> Option<Entry<U, V>>;
    fn remove(&mut self, k: u64) -> Option<Entry<U, V>>;
    fn values_mut<'a>(&'a mut self) -> impl Iterator<Item = &mut Entry<U, V>>
    where
        V: 'a;
    fn into_values(self) -> impl Iterator<Item = Entry<U, V>>;
    fn iter<'a>(&'a self) -> impl Iterator<Item = (&u64, &Entry<U, V>)>
    where
        V: 'a;
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (&u64, &mut Entry<U, V>)>
    where
        V: 'a;
}

macro_rules! impl_map {
    ($ty:ty) => {
        impl<U, V> Map<U, V> for $ty
        where
            U: PrimInt + Default + 'static,
        {
            fn len(&self) -> usize {
                self.len()
            }

            fn is_empty(&self) -> bool {
                self.is_empty()
            }

            fn get(&self, k: u64) -> Option<&Entry<U, V>> {
                self.get(&k)
            }

            fn get_mut(&mut self, k: u64) -> Option<&mut Entry<U, V>> {
                self.get_mut(&k)
            }

            fn clear(&mut self) {
                self.clear()
            }

            fn insert(&mut self, k: u64, v: Entry<U, V>) -> Option<Entry<U, V>> {
                self.insert(k, v)
            }

            fn remove(&mut self, k: u64) -> Option<Entry<U, V>> {
                self.remove(&k)
            }

            fn values_mut<'a>(&'a mut self) -> impl Iterator<Item = &mut Entry<U, V>>
            where
                V: 'a,
            {
                self.values_mut()
            }

            fn into_values(self) -> impl Iterator<Item = Entry<U, V>> {
                self.into_values()
            }

            fn iter<'a>(&'a self) -> impl Iterator<Item = (&u64, &Entry<U, V>)>
            where
                V: 'a,
            {
                self.iter()
            }

            fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (&u64, &mut Entry<U, V>)>
            where
                V: 'a,
            {
                self.iter_mut()
            }
        }
    };
}

impl_map!(BTreeMap<u64, Entry<U, V>>);
impl_map!(HashMap<u64, Entry<U, V>>);
