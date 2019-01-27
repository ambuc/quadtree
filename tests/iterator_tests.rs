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

mod util; // For unordered_elements_are.

// For testing .iter(), .iter_mut(), .regions(), .values(), .values_mut().
mod iterator_tests {
    use crate::util::unordered_elements_are;
    use quadtree_impl::Quadtree;

    fn mk_quadtree_for_iter_tests() -> Quadtree<i32, i8> {
        let mut q = Quadtree::<i32, i8>::new_with_anchor((-35, -35), 8);
        q.extend(vec![((0, -5), 10), ((-15, 20), -25), ((30, -35), 40)]);
        q
    }

    #[test]
    fn iter_all() {
        let q = mk_quadtree_for_iter_tests();

        debug_assert!(unordered_elements_are(
            q.iter(),
            vec![
                (&((-15, 20), (1, 1)), &-25),
                (&((0, -5), (1, 1)), &10),
                (&((30, -35), (1, 1)), &40)
            ]
        ));
    }

    #[test]
    fn iter_size_hint() {
        let q = mk_quadtree_for_iter_tests();

        let mut iter = q.iter();
        debug_assert_eq!(iter.size_hint(), (3, Some(3)));
        iter.next();
        debug_assert_eq!(iter.size_hint(), (2, Some(2)));
        iter.next();
        debug_assert_eq!(iter.size_hint(), (1, Some(1)));
        iter.next();
        debug_assert_eq!(iter.size_hint(), (0, Some(0)));
        iter.next();
        debug_assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    // The same as iter_all(), except we mutate each value by +1.
    #[test]
    fn iter_mut_all() {
        let mut q = mk_quadtree_for_iter_tests();

        for (_, v) in q.iter_mut() {
            *v += 1;
        }

        debug_assert!(unordered_elements_are(
            q.iter(),
            vec![
                (&((-15, 20), (1, 1)), &-24),
                (&((0, -5), (1, 1)), &11),
                (&((30, -35), (1, 1)), &41)
            ]
        ));
    }

    #[test]
    fn iter_mut_size_hint() {
        let mut q = mk_quadtree_for_iter_tests();

        let mut iter = q.iter_mut();
        debug_assert_eq!(iter.size_hint(), (3, Some(3)));
        iter.next();
        debug_assert_eq!(iter.size_hint(), (2, Some(2)));
        iter.next();
        debug_assert_eq!(iter.size_hint(), (1, Some(1)));
        iter.next();
        debug_assert_eq!(iter.size_hint(), (0, Some(0)));
        iter.next();
        debug_assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    #[test]
    fn iter_exact_size() {
        let q = mk_quadtree_for_iter_tests();
        let mut iter = q.iter();
        debug_assert_eq!(iter.len(), 3);
        iter.next();
        debug_assert_eq!(iter.len(), 2);
        iter.next();
        iter.next();
        debug_assert_eq!(iter.len(), 0);
        iter.next();
        debug_assert_eq!(iter.len(), 0);
    }

    #[test]
    fn iter_mut_exact_size() {
        let mut q = mk_quadtree_for_iter_tests();
        let mut iter = q.iter_mut();
        debug_assert_eq!(iter.len(), 3);
        iter.next();
        debug_assert_eq!(iter.len(), 2);
        iter.next();
        iter.next();
        debug_assert_eq!(iter.len(), 0);
        iter.next();
        debug_assert_eq!(iter.len(), 0);
    }

    #[test]
    fn regions() {
        let q = mk_quadtree_for_iter_tests();
        debug_assert!(unordered_elements_are(
            q.regions(),
            vec![
                &((0, -5), (1, 1)),
                &((-15, 20), (1, 1)),
                &((30, -35), (1, 1))
            ],
        ));
    }

    #[test]
    fn values() {
        let q = mk_quadtree_for_iter_tests();

        debug_assert!(unordered_elements_are(q.values(), vec![&10, &-25, &40]));
    }

    #[test]
    fn values_mut() {
        let mut q = mk_quadtree_for_iter_tests();

        for v in q.values_mut() {
            *v += 1;
        }

        debug_assert!(unordered_elements_are(q.values(), vec![&11, &-24, &41]));
    }

    #[test]
    fn into_iterator_consuming() {
        let q = mk_quadtree_for_iter_tests();
        let v: Vec<(((i32, i32), (i32, i32)), i8)> = q.into_iter().collect();

        debug_assert!(unordered_elements_are(
            v,
            vec![
                (((0, -5), (1, 1)), 10),
                (((-15, 20), (1, 1)), -25),
                (((30, -35), (1, 1)), 40),
            ],
        ));
    }

    #[test]
    fn into_iterator_reference() {
        let mut q = mk_quadtree_for_iter_tests();
        let iter: Vec<(&((i32, i32), (i32, i32)), &i8)> = (&q).into_iter().collect();
        debug_assert_eq!(iter.len(), 3);

        q.reset();
        debug_assert!(q.is_empty());
    }

    #[test]
    fn into_iterator_mutable_reference() {
        let mut q = mk_quadtree_for_iter_tests();

        for (_, v) in (&mut q).into_iter() {
            *v += 1;
        }

        debug_assert!(unordered_elements_are(
            q,
            vec![
                (((0, -5), (1, 1)), 11),
                (((-15, 20), (1, 1)), -24),
                (((30, -35), (1, 1)), 41),
            ],
        ));
    }
}
