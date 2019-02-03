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

use quadtree_impl::Quadtree;

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
        q.insert(
            /*anchor=*/ (0, 0),
            /*size=*/ (2, 3),
            /*value=*/ 4,
        );
        q.insert_pt(/*anchor=*/ (1, 1), /*value=*/ 3);

        // The full bounds of the region.
        q.insert(
            /*anchor=*/ (0, 0),
            /*size=*/ (4, 4),
            /*value=*/ 17,
        );
        // At (3, 3) but 1x1
        q.insert_pt(/*anchor=*/ (3, 3), /*value=*/ 19);
    }

    #[test]
    fn insert_unsucessful() {
        let mut q = Quadtree::<u32, u8>::new(2);
        // At (0, 0) and too large.
        q.insert(
            /*anchor=*/ (0, 0),
            /*size=*/ (5, 5),
            /*value=*/ 17,
        );
        // At (4, 4) but 1x1.
        q.insert_pt(/*anchor=*/ (4, 4), /*value=*/ 20);
    }

    #[test]
    fn insert_successful_outside_region() {
        // Since the region might overlap, insertion doesn't fail.
        let mut q = Quadtree::<u32, u16>::new_with_anchor((2, 2), 2);
        q.insert_pt(/*anchor=*/ (0, 0), /*value=*/ 25);
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
fn fill_quadrant() {
    let mut q = Quadtree::<u8, f64>::new(2);
    debug_assert!(q.is_empty());

    q.insert((0, 0), (2, 2), 49.17); // This should 100% fill one quadrant.
    debug_assert_eq!(q.len(), 1);
    debug_assert!(!q.is_empty());

    q.insert((2, 2), (2, 2), 71.94); // This should 100% fill one quadrant.
    debug_assert_eq!(q.len(), 2);
    debug_assert!(!q.is_empty());
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

// We should be able to store strings.
mod string {
    use super::*;

    #[test]
    fn quadtree_string() {
        let mut q = Quadtree::<u32, String>::new(4);
        q.insert((0, 0), (1, 1), "foo_bar_baz".to_string());

        let mut iter = q.query((0, 0), (1, 1));
        assert_eq!(iter.next().unwrap().value(), "foo_bar_baz");
    }

    #[test]
    fn quadtree_mut_string() {
        let mut q = Quadtree::<u32, String>::new(4);
        q.insert((0, 0), (1, 1), "hello ".to_string());
        q.modify((0, 0), (1, 1), |v| *v += "world");

        assert_eq!(
            q.query((0, 0), (1, 1)).next().unwrap().value(),
            "hello world"
        );
    }
}

// Test creating a complex struct (containing a string), embed that struct in the Quadtree, and
// then query for the struct by location and extract some public field from it.
#[test]
fn quadtree_struct() {
    #[derive(Clone)]
    struct Foo {
        pub baz: String,
    };
    let foo = Foo {
        baz: "baz".to_string(),
    };

    let mut q = Quadtree::<u32, Foo>::new(4);

    q.insert((0, 0), (1, 1), foo);

    assert_eq!(q.query((0, 0), (1, 1)).next().unwrap().value().baz, "baz");
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

        let entry_zero = q.query_pt((0, 0)).next().unwrap();
        debug_assert_eq!(entry_zero.region(), &((0, 0), (1, 1)));
        debug_assert_eq!(entry_zero.value(), &0);
        let entry_five = q.query_pt((2, 3)).next().unwrap();
        debug_assert_eq!(entry_five.region(), &((2, 3), (1, 1)));
        debug_assert_eq!(entry_five.value(), &5);
    }

    #[test]
    fn extend_with_points_and_regions() {
        let mut q = Quadtree::<u32, i8>::new(3);
        assert!(q.is_empty());

        q.extend(vec![(((0, 0), (1, 2)), 0), (((2, 3), (3, 4)), 5)]);

        debug_assert_eq!(q.len(), 2);

        dbg!(&q);

        debug_assert_eq!(q.query_pt((0, 0)).next().unwrap().value(), &0);
        debug_assert_eq!(q.query_pt((2, 3)).next().unwrap().value(), &5);
    }
}

#[test]
#[ignore]
fn debug() {
    let mut q = Quadtree::<u8, f64>::new(2);
    q.insert((0, 0), (2, 2), 1.35);
    q.insert((1, 1), (1, 1), 2.46);
    q.insert((1, 1), (2, 2), 3.69);
    q.insert((2, 2), (2, 2), 4.812);
    dbg!(&q);
}

#[test]
#[ignore]
fn test_print_quadtree() {
    use crate::util::print_quadtree;

    let mut q = quadtree_impl::Quadtree::<u8, f64>::new(3);
    q.insert((0, 0), (2, 2), 1.35);
    q.insert((2, 3), (1, 1), 2.46);
    q.insert((1, 1), (2, 2), 3.69);
    q.insert((2, 2), (4, 4), 4.812);
    q.insert((0, 4), (2, 3), 4.812);
    print_quadtree(&q);
}
