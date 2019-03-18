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

use quadtree_rs::Quadtree;

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
        let _q0 = Quadtree::<u32, i8>::new_with_anchor((1, 1).into(), 0);
        let _q1 = Quadtree::<u32, u32>::new_with_anchor((0, 510123).into(), 1);
        let _q2 = Quadtree::<u32, f64>::new_with_anchor((4009, 4009).into(), 2);
    }
}

#[test]
fn anchor() {
    debug_assert_eq!(Quadtree::<u32, u8>::new(0).anchor(), (0, 0).into());
    debug_assert_eq!(Quadtree::<u32, u8>::new(1).anchor(), (0, 0).into());
    debug_assert_eq!(Quadtree::<u32, u8>::new(2).anchor(), (0, 0).into());
    for x in [20, 49, 2013, 1, 0].iter() {
        for y in [10, 399, 20, 4, 397].iter() {
            debug_assert_eq!(
                Quadtree::<u32, u8>::new_with_anchor((*x, *y).into(), 2).anchor(),
                (*x, *y).into()
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
        let mut qt = Quadtree::<u32, u8>::new(2);
        qt.insert(
            (/*anchor=*/ (0, 0), /*size=*/ (2, 3)).into(),
            /*value=*/ 4,
        );
        qt.insert((/*anchor=*/ (1, 1), (1, 1)).into(), /*value=*/ 3);

        // The full bounds of the region.
        qt.insert(
            (/*anchor=*/ (0, 0), /*size=*/ (4, 4)).into(),
            /*value=*/ 17,
        );
        // At (3, 3) but 1x1
        qt.insert((/*anchor=*/ (3, 3), (1, 1)).into(), /*value=*/ 19);
    }

    #[test]
    fn insert_unsucessful() {
        let mut qt = Quadtree::<u32, u8>::new(2);
        // At (0, 0) and too large.
        qt.insert(
            (/*anchor=*/ (0, 0), /*size=*/ (5, 5)).into(),
            /*value=*/ 17,
        );
        // At (4, 4) but 1x1.
        qt.insert((/*anchor=*/ (4, 4), (1, 1)).into(), /*value=*/ 20);
    }

    #[test]
    fn insert_successful_outside_region() {
        // Since the region might overlap, insertion doesn't fail.
        let mut qt = Quadtree::<u32, u16>::new_with_anchor((2, 2).into(), 2);
        qt.insert((/*anchor=*/ (0, 0), (1, 1)).into(), /*value=*/ 25);
    }
}

#[test]
fn len() {
    let mut qt = Quadtree::<u32, u32>::new(4);
    debug_assert_eq!(qt.len(), 0);
    qt.insert(((0, 0), (1, 1)).into(), 2);
    debug_assert_eq!(qt.len(), 1);
    // Even if it's the same thing again.
    qt.insert(((0, 0), (1, 1)).into(), 2);
    debug_assert_eq!(qt.len(), 2);
    // Or if it's a point.
    qt.insert(((2, 3), (1, 1)).into(), 2);
    debug_assert_eq!(qt.len(), 3);
}

#[test]
fn fill_quadrant() {
    let mut qt = Quadtree::<u8, f64>::new(2);
    debug_assert!(qt.is_empty());

    qt.insert(((0, 0), (2, 2)).into(), 49.17); // This should 100% fill one quadrant.
    debug_assert_eq!(qt.len(), 1);
    debug_assert!(!qt.is_empty());

    qt.insert(((2, 2), (2, 2)).into(), 71.94); // This should 100% fill one quadrant.
    debug_assert_eq!(qt.len(), 2);
    debug_assert!(!qt.is_empty());
}

#[test]
fn is_empty() {
    let mut qt = Quadtree::<u32, u64>::new(2);
    debug_assert!(qt.is_empty());

    // Insert region
    qt.insert(((0, 0), (2, 2)).into(), 49);
    debug_assert!(!qt.is_empty());

    let mut q2 = Quadtree::<u32, u32>::new(4);
    debug_assert!(q2.is_empty());

    // Insert point
    q2.insert(((1, 1), (1, 1)).into(), 50);
    debug_assert!(!q2.is_empty());
}

#[test]
fn reset() {
    let mut qt = Quadtree::<u32, f32>::new(4);
    debug_assert!(qt.is_empty());

    qt.insert(((2, 2), (1, 1)).into(), 57.27);
    debug_assert!(!qt.is_empty());

    qt.reset();
    debug_assert!(qt.is_empty());
    debug_assert_eq!(qt.len(), 0);
}

// We should be able to store strings.
mod string {
    use super::*;

    #[test]
    fn quadtree_string() {
        let mut qt = Quadtree::<u32, String>::new(4);
        qt.insert(((0, 0), (1, 1)).into(), "foo_bar_baz".to_string());

        let mut iter = qt.query(((0, 0), (1, 1)).into());
        assert_eq!(iter.next().unwrap().value_ref(), "foo_bar_baz");
    }

    #[test]
    fn quadtree_mut_string() {
        let mut qt = Quadtree::<u32, String>::new(4);
        qt.insert(((0, 0), (1, 1)).into(), "hello ".to_string());
        qt.modify(((0, 0), (1, 1)).into(), |v| *v += "world");

        assert_eq!(
            qt.query(((0, 0), (1, 1)).into())
                .next()
                .unwrap()
                .value_ref(),
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

    let mut qt = Quadtree::<u32, Foo>::new(4);

    qt.insert(((0, 0), (1, 1)).into(), foo);

    assert_eq!(
        qt.query(((0, 0), (1, 1)).into())
            .next()
            .unwrap()
            .value_ref()
            .baz,
        "baz"
    );
}

// Since we implement Extend<((U, U), V)> for Quadtree<U, V>, test out .extend()ing with a real
// iterator.
mod extend {
    use super::*;

    #[test]
    fn extend_with_just_points() {
        let mut qt = Quadtree::<u32, i8>::new(4);
        assert!(qt.is_empty());

        qt.extend(vec![((0, 0), 0), ((2, 3), 5)]);

        debug_assert_eq!(qt.len(), 2);

        let entry_zero = qt.query(((0, 0), (1, 1)).into()).next().unwrap();
        let area_zero = entry_zero.area();
        debug_assert_eq!(area_zero.anchor(), (0, 0).into());
        debug_assert_eq!(area_zero.width(), 1);
        debug_assert_eq!(area_zero.height(), 1);

        debug_assert_eq!(entry_zero.value_ref(), &0);

        let entry_five = qt.query(((2, 3), (1, 1)).into()).next().unwrap();
        let area_five = entry_five.area();
        debug_assert_eq!(area_five.anchor(), (2, 3).into());
        debug_assert_eq!(area_five.width(), 1);
        debug_assert_eq!(area_five.height(), 1);

        debug_assert_eq!(entry_five.value_ref(), &5);
    }

    #[test]
    fn extend_with_points_and_regions() {
        let mut qt = Quadtree::<u32, i8>::new(3);
        assert!(qt.is_empty());

        qt.extend(vec![(((0, 0), (1, 2)), 0), (((2, 3), (3, 4)), 5)]);

        debug_assert_eq!(qt.len(), 2);

        debug_assert_eq!(
            qt.query(((0, 0), (1, 1)).into())
                .next()
                .unwrap()
                .value_ref(),
            &0
        );
        debug_assert_eq!(
            qt.query(((2, 3), (1, 1)).into())
                .next()
                .unwrap()
                .value_ref(),
            &5
        );
    }
}

mod delete {
    use super::*;

    #[test]
    fn delete_by_handle() {
        let mut qt = Quadtree::<u32, i8>::new(4);
        // We don't know the indices for any of these.
        qt.extend(vec![((0, 0), 0), ((2, 3), 5), ((2, 2), 7), ((1, 2), 9)]);
        debug_assert_eq!(qt.len(), 4);

        // But we will be sure to retain this one.
        let handle = qt.insert(((0, 0), (1, 1)).into(), 11);
        debug_assert_eq!(qt.len(), 5); // Insertion succeeded.

        // Check the returned entry.
        let entry = qt.delete_by_handle(handle).unwrap();
        let entry_area = entry.area();
        debug_assert_eq!(entry_area.anchor(), (0, 0).into());
        debug_assert_eq!(entry_area.width(), 1);
        debug_assert_eq!(entry_area.height(), 1);

        debug_assert_eq!(entry.value_ref(), &11);

        // And check that the tree is smaller now.
        debug_assert_eq!(qt.len(), 4); // Insertion succeeded.

        // And, check that queries over the previous area don't crash or return garbage indices.
        debug_assert_eq!(qt.query(((0, 0), (1, 1)).into()).count(), 1);
    }
}

#[test]
#[ignore]
fn debug() {
    let mut qt = Quadtree::<u8, f64>::new(2);
    qt.insert(((0, 0), (2, 2)).into(), 1.35);
    qt.insert(((1, 1), (1, 1)).into(), 2.46);
    qt.insert(((1, 1), (2, 2)).into(), 3.69);
    qt.insert(((2, 2), (2, 2)).into(), 4.812);
    dbg!(&qt);
}

#[test]
#[ignore]
fn test_print_quadtree() {
    use crate::util::print_quadtree;

    let mut qt = quadtree_rs::Quadtree::<u8, f64>::new(3);
    qt.insert(((0, 0), (2, 2)).into(), 1.35);
    qt.insert(((2, 3), (1, 1)).into(), 2.46);
    qt.insert(((1, 1), (2, 2)).into(), 3.69);
    qt.insert(((2, 2), (4, 4)).into(), 4.812);
    qt.insert(((0, 4), (2, 3)).into(), 4.812);
    print_quadtree(&qt);
}
