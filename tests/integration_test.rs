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

use quadtree_impl::Quadtree;

mod util;

mod new {
    use super::*;
    #[test]
    fn new_with_depth() {
        // None of these should crash.
        let _q0 = Quadtree::<u32, u8>::new(0);
        let _q1 = Quadtree::<u32, u64>::new(1);
        let _q2 = Quadtree::<u32, f32>::new(2);
    }

    #[test]
    fn new_with_anchor() {
        // None of these should crash.
        let _q0 = Quadtree::<u32, i8>::new_with_anchor((1, 1), 0);
        let _q1 = Quadtree::<u32, u32>::new_with_anchor((0, 510123), 1);
        let _q2 = Quadtree::<u32, f64>::new_with_anchor((4009, 4009), 2);
    }
}

#[test]
fn anchor() {
    debug_assert_eq!(Quadtree::<u32, u8>::new(0).anchor(), (0, 0));
    debug_assert_eq!(Quadtree::<u32, u8>::new(1).anchor(), (0, 0));
    debug_assert_eq!(Quadtree::<u32, u8>::new(2).anchor(), (0, 0));
    for x in [20, 49, 2013, 1, 0].iter() {
        for y in [10, 399, 20, 4, 397].iter() {
            debug_assert_eq!(
                Quadtree::<u32, u8>::new_with_anchor((*x, *y), 2).anchor(),
                (*x, *y)
            );
        }
    }
}

#[test]
fn width_and_height() {
    debug_assert_eq!(Quadtree::<u32, f32>::new(0).width(), 1);
    debug_assert_eq!(Quadtree::<u32, f32>::new(0).height(), 1);

    debug_assert_eq!(Quadtree::<u32, f32>::new(1).width(), 2);
    debug_assert_eq!(Quadtree::<u32, f32>::new(1).height(), 2);

    debug_assert_eq!(Quadtree::<u32, f32>::new(2).width(), 4);
    debug_assert_eq!(Quadtree::<u32, f32>::new(2).height(), 4);

    debug_assert_eq!(Quadtree::<u32, f32>::new(3).width(), 8);
    debug_assert_eq!(Quadtree::<u32, f32>::new(3).height(), 8);
}

mod insert {
    use super::*;

    #[test]
    fn insert_successful() {
        let mut q = Quadtree::<u32, u8>::new(2);
        debug_assert!(q.insert(
            /*anchor=*/ (0, 0),
            /*size=*/ (2, 3),
            /*value=*/ 4,
        ));
        debug_assert!(q.insert_pt(/*anchor=*/ (1, 1), /*value=*/ 3));

        // The full bounds of the region.
        debug_assert!(q.insert(
            /*anchor=*/ (0, 0),
            /*size=*/ (4, 4),
            /*value=*/ 17
        ));
        // At (3, 3) but 1x1
        debug_assert!(q.insert_pt(/*anchor=*/ (3, 3), /*value=*/ 19));
    }

    #[test]
    fn insert_unsucessful() {
        let mut q = Quadtree::<u32, u8>::new(2);
        // At (0, 0) and too large.
        debug_assert!(!q.insert(
            /*anchor=*/ (0, 0),
            /*size=*/ (5, 5),
            /*value=*/ 17
        ));
        // At (4, 4) but 1x1.
        debug_assert!(!q.insert_pt(/*anchor=*/ (4, 4), /*value=*/ 20));
    }

    #[test]
    fn insert_unsuccessful_outside_region() {
        let mut q = Quadtree::<u32, u16>::new_with_anchor((2, 2), 2);
        debug_assert!(!q.insert_pt(/*anchor=*/ (0, 0), /*value=*/ 25));
    }
}

#[test]
fn len() {
    let mut q = Quadtree::<u32, u32>::new(4);
    debug_assert_eq!(q.len(), 0);
    q.insert((0, 0), (1, 1), 2);
    debug_assert_eq!(q.len(), 1);
    // Even if it's the same thing again.
    q.insert((0, 0), (1, 1), 2);
    debug_assert_eq!(q.len(), 2);
    // Or if it's a point.
    q.insert_pt((2, 3), 2);
    debug_assert_eq!(q.len(), 3);
}

#[test]
fn is_empty() {
    let mut q = Quadtree::<u32, u64>::new(2);
    debug_assert!(q.is_empty());

    // Insert region
    q.insert((0, 0), (2, 2), 49);
    debug_assert!(!q.is_empty());

    let mut q2 = Quadtree::<u32, u32>::new(4);
    debug_assert!(q2.is_empty());

    // Insert point
    q2.insert_pt((1, 1), 50);
    debug_assert!(!q2.is_empty());
}

#[test]
fn reset() {
    let mut q = Quadtree::<u32, f32>::new(4);
    debug_assert!(q.is_empty());

    q.insert_pt((2, 2), 57.27);
    debug_assert!(!q.is_empty());

    q.reset();
    debug_assert!(q.is_empty());
    debug_assert_eq!(q.len(), 0);
}

mod query {
    use super::*;

    #[test]
    fn query_empty() {
        let q = Quadtree::<u32, u8>::new(2);
        let mut iter = q.query((0, 0), (4, 4));
        debug_assert_eq!(iter.size_hint(), (0, Some(0)));
        debug_assert_eq!(iter.next(), None);
        debug_assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    #[test]
    fn query_on_point() {
        let mut q = Quadtree::<u32, u8>::new(1);
        q.insert((0, 0), (1, 1), 49);

        // Requesting a region which does contain '49'.
        let mut iter1 = q.query((0, 0), (1, 1));
        debug_assert_eq!(iter1.size_hint(), (0, Some(1)));
        debug_assert_eq!(iter1.next(), Some((&((0, 0), (1, 1)), &49)));
        debug_assert_eq!(iter1.size_hint(), (0, Some(0)));
        debug_assert_eq!(iter1.next(), None);
        debug_assert_eq!(iter1.size_hint(), (0, Some(0)));

        // Requesting regions which don't contain '49'.
        let mut iter2 = q.query((0, 1), (1, 1));
        debug_assert_eq!(iter2.size_hint(), (0, Some(1)));
        debug_assert_eq!(iter2.next(), None);
        debug_assert_eq!(iter2.size_hint(), (0, Some(0)));

        let mut iter3 = q.query((1, 0), (1, 1));
        debug_assert_eq!(iter3.size_hint(), (0, Some(1)));
        debug_assert_eq!(iter3.next(), None);
        debug_assert_eq!(iter3.size_hint(), (0, Some(0)));

        let mut iter4 = q.query((1, 1), (1, 1));
        debug_assert_eq!(iter4.size_hint(), (0, Some(1)));
        debug_assert_eq!(iter4.next(), None);
        debug_assert_eq!(iter4.size_hint(), (0, Some(0)));
    }

    #[test]
    fn query_in_region() {
        use crate::util::unordered_elements_are;

        let mut q = Quadtree::<u32, u8>::new(4);
        //   0  1  2  3  4  5  6
        // 0 +--+--+--+--+--+--+
        //   |  |  |  |  |  |  |
        // 1 +--+--+--+--+--+--+
        //   |  |  |  |  |  |  |
        // 2 +--+--o o o o--+--+  o @ (2, 2)->(2x2) #10
        //   |  |   o o o   |  |  x @ (3, 3)->(2x2) #55
        // 3 +--+--o oxoxox x--+
        //   |  |   o oxox x   |
        // 4 +--+--o oxoxox x--+
        //   |  |  |   x x x   |
        // 5 +--+--+--x x x x--+
        //   |  |  |  |  |  |  |
        // 6 +--+--+--+--+--+--+
        debug_assert!(q.insert((2, 2), (2, 2), 10));
        debug_assert!(q.insert((3, 3), (2, 2), 55));

        let expected_ten = Some((&((2, 2), (2, 2)), &10));
        let expected_fifty_five = Some((&((3, 3), (2, 2)), &55));

        // Queries which turn up empty:
        let mut empty1 = q.query((1, 1), (1, 1));
        debug_assert_eq!(empty1.size_hint(), (0, Some(2)));
        debug_assert_eq!(empty1.next(), None);
        debug_assert_eq!(empty1.size_hint(), (0, Some(0)));

        let mut empty2 = q.query((0, 0), (2, 2));
        debug_assert_eq!(empty2.size_hint(), (0, Some(2)));
        debug_assert_eq!(empty2.next(), None);
        debug_assert_eq!(empty2.size_hint(), (0, Some(0)));

        let mut empty3 = q.query((0, 0), (6, 2));
        debug_assert_eq!(empty3.size_hint(), (0, Some(2)));
        debug_assert_eq!(empty3.next(), None);
        debug_assert_eq!(empty3.size_hint(), (0, Some(0)));

        let mut empty4 = q.query((0, 0), (2, 6));
        debug_assert_eq!(empty4.size_hint(), (0, Some(2)));
        debug_assert_eq!(empty4.next(), None);
        debug_assert_eq!(empty4.size_hint(), (0, Some(0)));

        // Queries which capture #10:
        let mut ten1 = q.query((2, 2), (1, 1));
        debug_assert_eq!(ten1.next(), expected_ten);
        debug_assert_eq!(ten1.next(), None);

        let mut ten2 = q.query((2, 3), (1, 1));
        debug_assert_eq!(ten2.next(), expected_ten);
        debug_assert_eq!(ten2.next(), None);

        let mut ten3 = q.query((3, 2), (1, 1));
        debug_assert_eq!(ten3.next(), expected_ten);
        debug_assert_eq!(ten3.next(), None);

        // Queries which capture #10 but are larger than 1x1.
        let mut ten4 = q.query((2, 2), (2, 1));
        debug_assert_eq!(ten4.next(), expected_ten);
        debug_assert_eq!(ten4.next(), None);

        let mut ten5 = q.query((2, 2), (1, 2));
        debug_assert_eq!(ten5.next(), expected_ten);
        debug_assert_eq!(ten5.next(), None);

        // Queries which capture #55:
        let mut fiftyfive1 = q.query((3, 4), (1, 1));
        debug_assert_eq!(fiftyfive1.next(), expected_fifty_five);
        debug_assert_eq!(fiftyfive1.next(), None);

        let mut fiftyfive2 = q.query((4, 3), (1, 1));
        debug_assert_eq!(fiftyfive2.next(), expected_fifty_five);
        debug_assert_eq!(fiftyfive2.next(), None);

        let mut fiftyfive3 = q.query((4, 4), (1, 1));
        debug_assert_eq!(fiftyfive3.next(), expected_fifty_five);
        debug_assert_eq!(fiftyfive3.next(), None);

        // Queries which capture #55 but are larger than 1x1.

        let mut fiftyfive4 = q.query((4, 3), (1, 2));
        debug_assert_eq!(fiftyfive4.next(), expected_fifty_five);
        debug_assert_eq!(fiftyfive4.next(), None);

        let mut fiftyfive5 = q.query((3, 4), (2, 2));
        debug_assert_eq!(fiftyfive5.next(), expected_fifty_five);
        debug_assert_eq!(fiftyfive5.next(), None);

        // Queries which capture both #10 and #55. Dunno in what order.

        debug_assert!(unordered_elements_are(
            q.query((3, 3), (1, 1)),
            vec![(&((2, 2), (2, 2)), &10), (&((3, 3), (2, 2)), &55)]
        ));

        debug_assert!(unordered_elements_are(
            q.query((3, 3), (3, 3)),
            vec![(&((2, 2), (2, 2)), &10), (&((3, 3), (2, 2)), &55)]
        ));

        debug_assert!(unordered_elements_are(
            q.query((0, 0), (6, 6)),
            vec![(&((2, 2), (2, 2)), &10), (&((3, 3), (2, 2)), &55)]
        ));

        debug_assert!(unordered_elements_are(
            q.query((2, 2), (6, 6)),
            vec![(&((2, 2), (2, 2)), &10), (&((3, 3), (2, 2)), &55)]
        ));

        debug_assert!(unordered_elements_are(
            q.query((2, 2), (2, 2)),
            vec![(&((2, 2), (2, 2)), &10), (&((3, 3), (2, 2)), &55)]
        ));
    }

    #[test]
    fn query_mut_empty() {
        let mut q = Quadtree::<u32, u8>::new(2);
        let mut iter = q.query_mut((0, 0), (4, 4));
        debug_assert_eq!(iter.next(), None);
    }

    #[test]
    fn query_mut() {
        let mut q = Quadtree::<u32, u8>::new(3);

        // Insert #49 at (0, 0)->1x1.
        q.insert((0, 0), (1, 1), 49);
        // Up it to 50,
        for (_, i) in q.query_mut((0, 0), (1, 1)) {
            *i += 1;
        }
        // And verify.
        let mut tmp_iter_1 = q.query((0, 0), (1, 1));
        debug_assert_eq!(tmp_iter_1.size_hint(), (0, Some(1)));
        debug_assert_eq!(tmp_iter_1.next(), Some((&((0, 0), (1, 1)), &50)));
        debug_assert_eq!(tmp_iter_1.size_hint(), (0, Some(0)));
        debug_assert_eq!(tmp_iter_1.next(), None);
        debug_assert_eq!(tmp_iter_1.size_hint(), (0, Some(0)));

        // Insert #17 at (2, 2)->3x3.
        debug_assert!(q.insert((2, 2), (3, 3), 17));
        // Up it to 18,
        for (_, i) in q.query_mut((1, 1), (2, 2)) {
            *i += 1;
        }
        // And verify.
        let mut tmp_iter_2 = q.query((2, 2), (1, 1));
        debug_assert_eq!(tmp_iter_2.size_hint(), (0, Some(2)));
        debug_assert_eq!(tmp_iter_2.next(), Some((&((2, 2), (3, 3)), &18)));
        debug_assert_eq!(tmp_iter_2.size_hint(), (0, Some(1)));
        debug_assert_eq!(tmp_iter_2.next(), None);
        debug_assert_eq!(tmp_iter_2.size_hint(), (0, Some(0)));

        // Reset everything in (0, 0)->6x6 to "0".
        for (_, i) in q.query_mut((0, 0), (6, 6)) {
            *i = 0;
        }
        // Every value is now 0.

        for (_, v) in q.query((0, 0), (6, 6)) {
            debug_assert_eq!(*v, 0);
        }
    }

    #[test]
    fn query_pt_mut() {
        let mut q = Quadtree::<u32, u8>::new(4);
        // Insert #27 at (0, 0)->1x1.
        debug_assert!(q.insert((0, 0), (1, 1), 27));

        let mut tmp_iter = q.query_pt_mut((0, 0));
        debug_assert_eq!(tmp_iter.next(), Some((&((0, 0), (1, 1)), &mut 27)));
        debug_assert_eq!(tmp_iter.next(), None);
    }
}

// We should be able to store strings.
mod string {
    use super::*;

    #[test]
    fn quadtree_string() {
        let mut q = Quadtree::<u32, String>::new(4);
        debug_assert!(q.insert((0, 0), (1, 1), "foo_bar_baz".to_string()));

        let mut iter = q.query((0, 0), (1, 1));
        assert_eq!(iter.next().map_or("", |(_, v)| v), "foo_bar_baz");
    }

    #[test]
    fn quadtree_mut_string() {
        let mut q = Quadtree::<u32, String>::new(4);
        debug_assert!(q.insert((0, 0), (1, 1), "hello ".to_string()));
        for (_, v) in q.query_mut((0, 0), (1, 1)) {
            *v += "world";
        }

        assert_eq!(
            q.query((0, 0), (1, 1)).next().map_or("", |(_, v)| v),
            "hello world"
        );
    }
}

// Test creating a complex struct (containing a string), embed that struct in the Quadtree, and
// then query for the struct by location and extract some public field from it.
#[test]
fn quadtree_struct() {
    struct Foo {
        pub baz: String,
    };
    let foo = Foo {
        baz: "baz".to_string(),
    };

    let mut q = Quadtree::<u32, Foo>::new(4);

    debug_assert!(q.insert((0, 0), (1, 1), foo));

    assert_eq!(
        q.query((0, 0), (1, 1))
            .next()
            .map_or(&"".to_string(), |(_, f)| &f.baz),
        "baz"
    );
}

// Since we implement Extend<((U, U), V)> for Quadtree<U, V>, test out .extend()ing with a real
// iterator.
mod extend {
    use super::*;

    #[test]
    fn extend_with_just_points() {
        let mut q = Quadtree::<u32, i8>::new(4);
        assert!(q.is_empty());

        q.extend(vec![((0, 0), 0), ((2, 3), 5)]);

        debug_assert_eq!(q.len(), 2);

        debug_assert_eq!(q.query_pt((0, 0)).next(), Some((&((0, 0), (1, 1)), &0)));
        debug_assert_eq!(q.query_pt((2, 3)).next(), Some((&((2, 3), (1, 1)), &5)));
    }

    #[test]
    fn extend_with_points_and_regions() {
        let mut q = Quadtree::<u32, i8>::new(4);
        assert!(q.is_empty());

        q.extend(vec![(((0, 0), (1, 2)), 0), (((2, 3), (3, 4)), 5)]);

        debug_assert_eq!(q.len(), 2);

        debug_assert_eq!(q.query_pt((0, 0)).next(), Some((&((0, 0), (1, 2)), &0)));
        debug_assert_eq!(q.query_pt((2, 3)).next(), Some((&((2, 3), (3, 4)), &5)));
    }
}

// Test .iter(), .iter_mut().
mod iterator {
    use super::*;

    fn mk_quadtree_for_iter_tests() -> Quadtree<i32, i8> {
        let mut q = Quadtree::<i32, i8>::new_with_anchor((-35, -35), 8);
        q.extend(vec![((0, -5), 10), ((-15, 20), -25), ((30, -35), 40)]);
        q
    }

    #[test]
    fn iter_all() {
        use crate::util::unordered_elements_are;
        let mut q = mk_quadtree_for_iter_tests();

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
        let mut q = mk_quadtree_for_iter_tests();

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
        use crate::util::unordered_elements_are;

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
        let mut q = mk_quadtree_for_iter_tests();
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
}
