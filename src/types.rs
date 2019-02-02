// Copyright 2019 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! All the return types (Iter, IterMut, Query, etc.)

// d888888b d888888b d88888b d8888b.
//   `88'   `~~88~~' 88'     88  `8D
//    88       88    88ooooo 88oobY'
//    88       88    88~~~~~ 88`8b
//   .88.      88    88.     88 `88.
// Y888888P    YP    Y88888P 88   YD
use crate::geometry::area::{Area, AreaType};
use crate::qtinner::QTInner;
use num::PrimInt;
use std::collections::HashMap;
use std::iter::FusedIterator;
use uuid::Uuid;

// TODO(ambuc): Is it possible to collapse the .next() logic between this and IterMut and IntoIter?
/// An iterator over all regions and values of a [`Quadtree`].
///
/// This struct is created by the [`iter`] method on [`Quadtree`].
///
/// [`iter`]: ../struct.Quadtree.html#method.iter
/// [`Quadtree`]: ../struct.Quadtree.html
#[derive(Clone, Debug)]
pub struct Iter<'a, U, V>
where
    U: PrimInt,
{
    uuid_stack: Vec<Uuid>,
    qt_stack: Vec<&'a QTInner<U>>,
    remaining: usize,
    store: &'a HashMap<Uuid, (Area<U>, V)>,
}

impl<'a, U, V> Iter<'a, U, V>
where
    U: PrimInt,
{
    pub(crate) fn new(
        qt: &'a QTInner<U>,
        store: &'a HashMap<Uuid, (Area<U>, V)>,
    ) -> Iter<'a, U, V> {
        Iter {
            uuid_stack: vec![],
            qt_stack: vec![qt],
            remaining: qt.len(),
            store,
        }
    }
}

impl<'a, U, V> Iterator for Iter<'a, U, V>
where
    U: PrimInt,
{
    type Item = (&'a AreaType<U>, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Check the uuid_stack.
        if let Some(uuid) = self.uuid_stack.pop() {
            self.remaining -= 1;
            let result = self.store.get(&uuid).unwrap();
            return Some((&result.0.inner(), &result.1));
        }

        // Then check the qt_stack.
        if let Some(qt) = self.qt_stack.pop() {
            // Push my regions onto the region stack
            for uuid in qt.kept_uuids.iter() {
                self.uuid_stack.push(uuid.clone());
            }
            // Push my subquadrants onto the qt_stack too.
            if let Some(sqs) = qt.subquadrants.as_ref() {
                for sq in sqs.iter() {
                    self.qt_stack.push(sq);
                }
            }
            return self.next();
        }

        // Else there's nothing left to search.
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a, U, V> FusedIterator for Iter<'a, U, V> where U: PrimInt {}

impl<'a, U, V> ExactSizeIterator for Iter<'a, U, V>
where
    U: PrimInt,
{
    fn len(&self) -> usize {
        self.remaining
    }
}

// // d888888b d888888b d88888b d8888b. .88b  d88. db    db d888888b
// //   `88'   `~~88~~' 88'     88  `8D 88'YbdP`88 88    88 `~~88~~'
// //    88       88    88ooooo 88oobY' 88  88  88 88    88    88
// //    88       88    88~~~~~ 88`8b   88  88  88 88    88    88
// //   .88.      88    88.     88 `88. 88  88  88 88b  d88    88
// // Y888888P    YP    Y88888P 88   YD YP  YP  YP ~Y8888P'    YP
//
// /// A mutable iterator over all regions and values of a [`Quadtree`].
// ///
// /// This struct is created by the [`iter_mut`] method on [`Quadtree`].
// ///
// /// [`iter_mut`]: ../struct.Quadtree.html#method.iter_mut
// /// [`Quadtree`]: ../struct.Quadtree.html
// #[derive(Debug)]
// pub struct IterMut<'a, U, V>
// where
//     U: PrimInt,
// {
//     uuid_stack: Vec<Uuid>,
//     qt_stack: Vec<&'a mut QTInner<U>>,
//     remaining: usize,
//     store: &'a mut HashMap<Uuid, (Area<U>, V)>,
// }
//
// impl<'a, U, V> IterMut<'a, U, V>
// where
//     U: PrimInt,
// {
//     pub(crate) fn new(
//         qt: &'a mut QTInner<U>,
//         store: &'a mut HashMap<Uuid, (Area<U>, V)>,
//     ) -> IterMut<'a, U, V> {
//         let len = qt.len();
//         IterMut {
//             uuid_stack: vec![],
//             qt_stack: vec![qt],
//             remaining: len,
//             store,
//         }
//     }
//
//     fn return_uuid(&'a mut self, uuid: Uuid) -> (&'a AreaType<U>, &'a mut V) {
//         match self.store.entry(uuid) {
//             std::collections::hash_map::Entry::Occupied(oc) => {
//                 let o: &'a mut (Area<U>, V) = oc.into_mut();
//                 return (o.0.inner(), &mut o.1);
//             }
//             _ => panic!("?"),
//         }
//     }
// }
//
// impl<'a, U, V> Iterator for IterMut<'a, U, V>
// where
//     U: PrimInt,
// {
//     type Item = (&'a AreaType<U>, &'a mut V);
//
//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         // Check the uuid_stack.
//         if let Some(uuid) = self.uuid_stack.pop() {
//             self.remaining -= 1;
//             return Some(self.return_uuid(uuid));
//         }
//
//         // Then check the qt_stack.
//         if let Some(qt) = self.qt_stack.pop() {
//             // Push my regions onto the region stack
//             for uuid in qt.kept_uuids.iter_mut() {
//                 self.uuid_stack.push(uuid.clone());
//             }
//             // Push my subquadrants onto the qt_stack too.
//             if let Some(sqs) = qt.subquadrants.as_mut() {
//                 for sq in sqs.iter_mut() {
//                     self.qt_stack.push(sq);
//                 }
//             }
//             return self.next();
//         }
//
//         // Else there's nothing left to search.
//         None
//     }
//
//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         (self.remaining, Some(self.remaining))
//     }
// }
//
// impl<'a, U, V> FusedIterator for IterMut<'a, U, V>
// where
//     U: PrimInt,
// {
// }
//
// impl<'a, U, V> ExactSizeIterator for IterMut<'a, U, V>
// where
//     U: PrimInt,
// {
//     fn len(&self) -> usize {
//         self.remaining
//     }
// }

// d888888b d8b   db d888888b  .d88b.  d888888b d888888b d88888b d8888b.
//   `88'   888o  88 `~~88~~' .8P  Y8.   `88'   `~~88~~' 88'     88  `8D
//    88    88V8o 88    88    88    88    88       88    88ooooo 88oobY'
//    88    88 V8o88    88    88    88    88       88    88~~~~~ 88`8b
//   .88.   88  V888    88    `8b  d8'   .88.      88    88.     88 `88.
// Y888888P VP   V8P    YP     `Y88P'  Y888888P    YP    Y88888P 88   YD

/// A consuming iterator over all region/value pairs held in a [`Quadtree`].
///
/// TODO(ambuc): How is this created? `.into_iter()`? Find the right URL for it, if it's part of
/// IntoIterator.
///
/// [`Quadtree`]: ../struct.Quadtree.html
#[derive(Clone, Debug)]
pub struct IntoIter<U, V>
where
    U: PrimInt,
{
    uuid_stack: Vec<Uuid>,
    qt_stack: Vec<QTInner<U>>,
    remaining: usize,
    store: HashMap<Uuid, (Area<U>, V)>,
}

impl<U, V> IntoIter<U, V>
where
    U: PrimInt,
{
    pub(crate) fn new(qt: QTInner<U>, store: HashMap<Uuid, (Area<U>, V)>) -> IntoIter<U, V> {
        let len = qt.len();
        IntoIter {
            uuid_stack: Vec::new(),
            qt_stack: vec![qt],
            remaining: len,
            store,
        }
    }
}

impl<U, V> Iterator for IntoIter<U, V>
where
    U: PrimInt,
{
    type Item = (AreaType<U>, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Check the uuid_stack.
        if let Some(uuid) = self.uuid_stack.pop() {
            self.remaining -= 1;
            let (_uuid, kv) = self.store.remove_entry(&uuid).unwrap();
            return Some((*kv.0.inner(), kv.1));
        }
        // Then check the qt_stack.
        if let Some(qt) = self.qt_stack.pop() {
            // Push my regions onto the region stack
            for uuid in qt.kept_uuids.into_iter() {
                self.uuid_stack.push(uuid.clone());
            }
            // Push my subquadrants onto the qt_stack too.
            if let Some([a, b, c, d]) = qt.subquadrants {
                self.qt_stack.push(*a);
                self.qt_stack.push(*b);
                self.qt_stack.push(*c);
                self.qt_stack.push(*d);
            }
            return self.next();
        }

        // Else there's nothing left to search.
        None
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<U, V> ExactSizeIterator for IntoIter<U, V>
where
    U: PrimInt,
{
    #[inline]
    fn len(&self) -> usize {
        self.remaining
    }
}

impl<U, V> FusedIterator for IntoIter<U, V> where U: PrimInt {}

//  .d88b.  db    db d88888b d8888b. db    db
// .8P  Y8. 88    88 88'     88  `8D `8b  d8'
// 88    88 88    88 88ooooo 88oobY'  `8bd8'
// 88    88 88    88 88~~~~~ 88`8b      88
// `8P  d8' 88b  d88 88.     88 `88.    88
//  `Y88'Y8 ~Y8888P' Y88888P 88   YD    YP

/// An iterator over the regions and values of a [`Quadtree`].
///
/// This struct is created by the [`query`] or [`query_pt`] methods on [`Quadtree`].
///
/// [`query`]: ../struct.Quadtree.html#method.query
/// [`query_pt`]: ../struct.Quadtree.html#method.query_pt
/// [`Quadtree`]: ../struct.Quadtree.html
// TODO(ambuc): This is pretty inefficient at the moment -- it starts at the top level and checks
// everything. This has to be fixed before v1.0.0. (Same for QueryMut.)
#[derive(Clone, Debug)]
pub struct Query<'a, U, V>
where
    U: PrimInt,
{
    pub(crate) query_region: Area<U>,
    pub(crate) inner: Iter<'a, U, V>,
}

impl<'a, U, V> Iterator for Query<'a, U, V>
where
    U: PrimInt,
{
    type Item = (&'a AreaType<U>, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map_or(None, |(k, v)| {
            if self.query_region.intersects(k.into()) {
                Some((k, v))
            } else {
                self.next()
            }
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, U, V> FusedIterator for Query<'a, U, V> where U: PrimInt {}

// //   .d88b.  db    db d88888b d8888b. db    db .88b  d88. db    db d888888b
// //  .8P  Y8. 88    88 88'     88  `8D `8b  d8' 88'YbdP`88 88    88 `~~88~~'
// //  88    88 88    88 88ooooo 88oobY'  `8bd8'  88  88  88 88    88    88
// //  88    88 88    88 88~~~~~ 88`8b      88    88  88  88 88    88    88
// //  `8P  d8' 88b  d88 88.     88 `88.    88    88  88  88 88b  d88    88
// //   `Y88'Y8 ~Y8888P' Y88888P 88   YD    YP    YP  YP  YP ~Y8888P'    YP
//
// /// A mutable iterator over the regions and values of a [`Quadtree`].
// ///
// /// This struct is created by the [`query_mut`] or [`query_pt_mut`] methods on [`Quadtree`].
// ///
// /// [`query_mut`]: ../struct.Quadtree.html#method.query_mut
// /// [`query_pt_mut`]: ../struct.Quadtree.html#method.query_pt_mut
// /// [`Quadtree`]: ../struct.Quadtree.html
// pub struct QueryMut<'a, U, V>
// where
//     U: PrimInt,
// {
//     pub(crate) query_region: Area<U>,
//     pub(crate) inner: IterMut<'a, U, V>,
// }
//
// impl<'a, U, V> Iterator for QueryMut<'a, U, V>
// where
//     U: PrimInt,
// {
//     type Item = (&'a AreaType<U>, &'a mut V);
//
//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         self.inner.next().map_or(None, |(k, v)| {
//             if self.query_region.intersects(k.into()) {
//                 Some((k, v))
//             } else {
//                 self.next()
//             }
//         })
//     }
//
//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.inner.size_hint()
//     }
// }
//
// impl<'a, U, V> FusedIterator for QueryMut<'a, U, V>
// where
//     U: PrimInt,
// {
// }

// d8888b. d88888b  d888b  d888888b  .d88b.  d8b   db .d8888.
// 88  `8D 88'     88' Y8b   `88'   .8P  Y8. 888o  88 88'  YP
// 88oobY' 88ooooo 88         88    88    88 88V8o 88 `8bo.
// 88`8b   88~~~~~ 88  ooo    88    88    88 88 V8o88   `Y8b.
// 88 `88. 88.     88. ~8~   .88.   `8b  d8' 88  V888 db   8D
// 88   YD Y88888P  Y888P  Y888888P  `Y88P'  VP   V8P `8888Y'

/// An iterator over the regions held within a [`Quadtree`].
///
/// This struct is created by the [`regions`] method on [`Quadtree`].
///
/// [`regions`]: ../struct.Quadtree.html#method.regions
/// [`Quadtree`]: ../struct.Quadtree.html
#[derive(Clone, Debug)]
pub struct Regions<'a, U, V>
where
    U: PrimInt,
{
    pub(crate) inner: Iter<'a, U, V>,
}

impl<'a, U, V> Iterator for Regions<'a, U, V>
where
    U: PrimInt,
{
    type Item = (&'a AreaType<U>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map_or(None, |(k, _v)| Some(k))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, U, V> FusedIterator for Regions<'a, U, V> where U: PrimInt {}

impl<'a, U, V> ExactSizeIterator for Regions<'a, U, V>
where
    U: PrimInt,
{
    fn len(&self) -> usize {
        self.inner.len()
    }
}

// db    db  .d8b.  db      db    db d88888b .d8888.
// 88    88 d8' `8b 88      88    88 88'     88'  YP
// Y8    8P 88ooo88 88      88    88 88ooooo `8bo.
// `8b  d8' 88~~~88 88      88    88 88~~~~~   `Y8b.
//  `8bd8'  88   88 88booo. 88b  d88 88.     db   8D
//    YP    YP   YP Y88888P ~Y8888P' Y88888P `8888Y'

/// An iterator over the values held within a [`Quadtree`].
///
/// This struct is created by the [`values`] method on [`Quadtree`].
///
/// [`values`]: ../struct.Quadtree.html#method.values
/// [`Quadtree`]: ../struct.Quadtree.html
#[derive(Clone, Debug)]
pub struct Values<'a, U, V>
where
    U: PrimInt,
{
    pub(crate) inner: Iter<'a, U, V>,
}

impl<'a, U, V> Iterator for Values<'a, U, V>
where
    U: PrimInt,
{
    type Item = (&'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map_or(None, |(_k, v)| Some(v))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, U, V> FusedIterator for Values<'a, U, V> where U: PrimInt {}

impl<'a, U, V> ExactSizeIterator for Values<'a, U, V>
where
    U: PrimInt,
{
    fn len(&self) -> usize {
        self.inner.len()
    }
}

// // db    db  .d8b.  db      db    db d88888b .d8888. .88b  d88. db    db d888888b
// // 88    88 d8' `8b 88      88    88 88'     88'  YP 88'YbdP`88 88    88 `~~88~~'
// // Y8    8P 88ooo88 88      88    88 88ooooo `8bo.   88  88  88 88    88    88
// // `8b  d8' 88~~~88 88      88    88 88~~~~~   `Y8b. 88  88  88 88    88    88
// //  `8bd8'  88   88 88booo. 88b  d88 88.     db   8D 88  88  88 88b  d88    88
// //    YP    YP   YP Y88888P ~Y8888P' Y88888P `8888Y' YP  YP  YP ~Y8888P'    YP
//
// /// A mutable iterator over the values held within a [`Quadtree`].
// ///
// /// This struct is created by the [`values_mut`] method on [`Quadtree`].
// ///
// /// [`values_mut`]: ../struct.Quadtree.html#method.values_mut
// /// [`Quadtree`]: ../struct.Quadtree.html
// #[derive(Debug)]
// pub struct ValuesMut<'a, U, V>
// where
//     U: PrimInt,
// {
//     pub(crate) inner: IterMut<'a, U, V>,
// }
//
// impl<'a, U, V> Iterator for ValuesMut<'a, U, V>
// where
//     U: PrimInt,
// {
//     type Item = (&'a mut V);
//
//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         self.inner.next().map_or(None, |(_k, v)| Some(v))
//     }
//
//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.inner.size_hint()
//     }
// }
//
// impl<'a, U, V> FusedIterator for ValuesMut<'a, U, V>
// where
//     U: PrimInt,
// {
// }
//
// impl<'a, U, V> ExactSizeIterator for ValuesMut<'a, U, V>
// where
//     U: PrimInt,
// {
//     fn len(&self) -> usize {
//         self.inner.len()
//     }
// }
