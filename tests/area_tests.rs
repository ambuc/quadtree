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

mod area_tests {
    use quadtree_rs::area::{Area, AreaBuilder};

    mod builder {
        use super::*;

        #[test]
        fn builder() {
            let a: Area<i8> = AreaBuilder::default()
                .anchor((0, 0).into())
                .dimensions((2, 2))
                .build()
                .unwrap();
            debug_assert_eq!(a.width(), 2);
        }
    }

    #[test]
    fn bad_dims() {
        for dims in [(-1, 4), (1, -4), (0, 4), (1, 0)].iter() {
            debug_assert!(AreaBuilder::default()
                .anchor((0, 0).into())
                .dimensions(*dims)
                .build()
                .is_err());
        }
    }

    #[test]
    fn point_in_all_quadrants() {
        for p in [(1, 1), (-1, 1), (1, -1), (-1, -1)].iter() {
            let _a: Area<i8> = AreaBuilder::default().anchor(p.into()).build().unwrap();
        }
    }

    #[test]
    fn properties() {
        let a = AreaBuilder::default()
            .anchor((3, 4).into())
            .dimensions((5, 7))
            .build()
            .unwrap();

        debug_assert_eq!(a.anchor(), (3, 4).into());
        debug_assert_eq!(a.width(), 5);
        debug_assert_eq!(a.height(), 7);

        debug_assert_eq!(a.left_edge(), 3);
        debug_assert_eq!(a.top_edge(), 4);
        debug_assert_eq!(a.right_edge(), /*3+5*/ 8);
        debug_assert_eq!(a.bottom_edge(), /*4+7*/ 11);
    }

    // Just positive values.
    mod contains_a {
        use super::*;

        //   0  1  2  3  4
        // 0 +--+--+--+--+
        //   |  |  |  |  |
        // 1 +--aaaaaaa--+
        //   |  aaaaaaa  |
        // 2 +--aaaaaaa--+
        //   |  aaaaaaa  |
        // 3 +--aaaaaaa--+
        //   |  |  |  |  |
        // 4 +--+--+--+--+

        fn test_area() -> Area<u8> {
            AreaBuilder::default()
                .anchor((1, 1).into())
                .dimensions((2, 2))
                .build()
                .unwrap()
        }

        #[test]
        fn all_component_1x1s() {
            let a = test_area();

            debug_assert!(a.contains(
                AreaBuilder::default()
                    .anchor((1, 1).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.contains(
                AreaBuilder::default()
                    .anchor((1, 2).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.contains(
                AreaBuilder::default()
                    .anchor((2, 1).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.contains(
                AreaBuilder::default()
                    .anchor((2, 2).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
        }

        #[test]
        fn contains_self() {
            let a = test_area();

            debug_assert!(a.contains(
                AreaBuilder::default()
                    .anchor((1, 1).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            ));
        }

        #[test]
        fn no_neighboring_1x1s() {
            let a = test_area();

            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((1, 0).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((2, 0).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((3, 0).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((4, 0).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((0, 3).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((1, 3).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((2, 3).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((3, 3).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((4, 3).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((0, 1).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((0, 2).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((0, 3).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((3, 1).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((3, 2).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((3, 3).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
        }

        #[test]
        fn no_overlapping_2x2s() {
            let a = test_area();

            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((2, 2).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            ));
        }

        #[test]
        fn no_overlapping_3x3s() {
            let a = test_area();

            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .dimensions((3, 3))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((1, 0).into())
                    .dimensions((3, 3))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((1, 1).into())
                    .dimensions((3, 3))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((1, 1).into())
                    .dimensions((3, 3))
                    .build()
                    .unwrap()
            ));
        }

        #[test]
        fn contains_pt() {
            let a = test_area();

            // DOES contain:
            debug_assert!(a.contains_pt((1, 1).into()));
            debug_assert!(a.contains_pt((1, 2).into()));
            debug_assert!(a.contains_pt((2, 1).into()));
            debug_assert!(a.contains_pt((2, 2).into()));

            // Does NOT contain:
            debug_assert!(!a.contains_pt((0, 0).into()));
            debug_assert!(!a.contains_pt((0, 1).into()));
            debug_assert!(!a.contains_pt((0, 2).into()));
            debug_assert!(!a.contains_pt((0, 3).into()));
            debug_assert!(!a.contains_pt((1, 0).into()));
            debug_assert!(!a.contains_pt((2, 0).into()));
            debug_assert!(!a.contains_pt((3, 0).into()));
            debug_assert!(!a.contains_pt((3, 0).into()));
            debug_assert!(!a.contains_pt((3, 1).into()));
            debug_assert!(!a.contains_pt((3, 2).into()));
            debug_assert!(!a.contains_pt((3, 3).into()));
        }
    }

    // Positive and negative values.
    mod contains_b {
        use super::*;

        //  -2 -1  0  1  2
        //-2 +--+--+--+--+
        //   |  |  |  |  |
        //-1 +--aaaaaaa--+
        //   |  aaaaaaa  |
        // 0 +--aaaaaaa--+
        //   |  aaaaaaa  |
        // 1 +--aaaaaaa--+
        //   |  |  |  |  |
        // 2 +--+--+--+--+

        fn test_area() -> Area<i8> {
            AreaBuilder::default()
                .anchor((-1, -1).into())
                .dimensions((2, 2))
                .build()
                .unwrap()
        }

        #[test]
        fn contains_one() {
            let a = test_area();

            debug_assert!(a.contains(
                AreaBuilder::default()
                    .anchor((-1, -1).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.contains(
                AreaBuilder::default()
                    .anchor((0, -1).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.contains(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.contains(
                AreaBuilder::default()
                    .anchor((-1, 0).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
        }

        #[test]
        fn contains_self() {
            let a = test_area();

            debug_assert!(a.contains(
                AreaBuilder::default()
                    .anchor((-1, -1).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            ));
        }

        #[test]
        fn no_neighboring_1x1s() {
            let a = test_area();

            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((-2, -2).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((-2, -1).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((-2, 0).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((-2, 1).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((-2, 2).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((-1, 2).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((0, 2).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((1, 2).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((2, 2).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((2, 1).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((2, 0).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((2, -1).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((2, -2).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((1, -2).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((0, -2).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((-1, -2).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
        }

        #[test]
        fn no_overlapping_2x2s() {
            let a = test_area();

            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((2, 2).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((-2, -2).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            ));
        }

        #[test]
        fn no_overlapping_3x3s() {
            let a = test_area();

            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .dimensions((3, 3))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((1, 0).into())
                    .dimensions((3, 3))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((-1, -1).into())
                    .dimensions((3, 3))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((-1, 1).into())
                    .dimensions((3, 3))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((-2, 1).into())
                    .dimensions((3, 3))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.contains(
                AreaBuilder::default()
                    .anchor((-2, -2).into())
                    .dimensions((3, 3))
                    .build()
                    .unwrap()
            ));
        }

        #[test]
        fn contains_pt() {
            let a = test_area();

            // DOES contain:
            debug_assert!(a.contains_pt((-1, -1).into()));
            debug_assert!(a.contains_pt((-1, 0).into()));
            debug_assert!(a.contains_pt((0, -1).into()));
            debug_assert!(a.contains_pt((0, 0).into()));

            // Does NOT contain:
            debug_assert!(!a.contains_pt((-2, -2).into()));
            debug_assert!(!a.contains_pt((-2, -1).into()));
            debug_assert!(!a.contains_pt((-2, 0).into()));
            debug_assert!(!a.contains_pt((-2, 1).into()));
            debug_assert!(!a.contains_pt((-2, 2).into()));
            debug_assert!(!a.contains_pt((-1, 2).into()));
            debug_assert!(!a.contains_pt((0, 2).into()));
            debug_assert!(!a.contains_pt((1, 2).into()));
            debug_assert!(!a.contains_pt((2, 2).into()));
            debug_assert!(!a.contains_pt((2, 1).into()));
            debug_assert!(!a.contains_pt((2, 0).into()));
            debug_assert!(!a.contains_pt((2, -1).into()));
            debug_assert!(!a.contains_pt((2, -2).into()));
            debug_assert!(!a.contains_pt((1, -2).into()));
            debug_assert!(!a.contains_pt((0, -2).into()));
            debug_assert!(!a.contains_pt((-1, -2).into()));
        }
    }

    // Just positive values.
    mod intersects_a {
        use super::*;

        //   0  1  2  3  4  5  6
        // 0 +--+--+--+--+--+--+
        //   |  |  |  |  |  |  |
        // 1 +--+--+--+--+--+--+
        //   |  |  |  |  |  |  |
        // 2 +--+--aaaaaaa--+--+
        //   |  |  aaaaaaa  |  |
        // 3 +--+--aaaaaaa--+--+
        //   |  |  aaaaaaa  |  |
        // 4 +--+--aaaaaaa--+--+
        //   |  |  |  |  |  |  |
        // 5 +--+--+--+--+--+--+
        //   |  |  |  |  |  |  |
        // 6 +--+--+--+--+--+--+

        fn test_area() -> Area<u8> {
            AreaBuilder::default()
                .anchor((2, 2).into())
                .dimensions((2, 2))
                .build()
                .unwrap()
        }

        // All the 1x1s obviously contains.
        #[test]
        fn area_1x1() {
            let a = test_area();

            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((2, 2).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((2, 3).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((3, 2).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((3, 3).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
        }

        // And the one 2x2 obviously contained.
        #[test]
        fn area_2x2() {
            let a = test_area();

            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((2, 2).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            ));
        }

        // But a single edge shared is not enough.
        #[test]
        fn area_with_only_a_single_shared_edge() {
            let a = test_area();

            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((1, 1).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((1, 1).into())
                    .dimensions((2, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((1, 1).into())
                    .dimensions((4, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((2, 1).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((3, 1).into())
                    .dimensions((2, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((4, 1).into())
                    .dimensions((2, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((1, 1).into())
                    .dimensions((1, 2))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((1, 2).into())
                    .dimensions((1, 2))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((1, 3).into())
                    .dimensions((1, 2))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((1, 4).into())
                    .dimensions((1, 2))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((2, 4).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((3, 4).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((4, 4).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
        }

        // But intersecting a 1x1 region counts.
        #[test]
        fn area_with_a_1x1_overlap() {
            let a = test_area();

            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((1, 1).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .dimensions((3, 3))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((3, 3).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((1, 3).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            ));
        }

        #[test]
        fn regression_test() {
            let a: Area<u8> = AreaBuilder::default()
                .anchor((3, 3).into())
                .dimensions((2, 2))
                .build()
                .unwrap();
            let b: Area<u8> = AreaBuilder::default()
                .anchor((0, 0).into())
                .dimensions((6, 6))
                .build()
                .unwrap();

            debug_assert!(b.intersects(a));
            debug_assert!(a.intersects(b));
        }
    }

    // Positive and negative values.
    mod intersects_b {
        use super::*;

        //  -3 -2 -1  0  1  2  3
        //-3 +--+--+--+--+--+--+
        //   |  |  |  |  |  |  |
        //-2 +--+--+--+--+--+--+
        //   |  |  |  |  |  |  |
        //-1 +--+--aaaaaaa--+--+
        //   |  |  aaaaaaa  |  |
        // 0 +--+--aaaaaaa--+--+
        //   |  |  aaaaaaa  |  |
        // 1 +--+--aaaaaaa--+--+
        //   |  |  |  |  |  |  |
        // 2 +--+--+--+--+--+--+
        //   |  |  |  |  |  |  |
        // 3 +--+--+--+--+--+--+

        fn test_area() -> Area<i8> {
            AreaBuilder::default()
                .anchor((-1, -1).into())
                .dimensions((2, 2))
                .build()
                .unwrap()
        }

        #[test]
        fn area_1x1() {
            let a = test_area();
            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((-1, -1).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((-1, 0).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((0, -1).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
        }

        #[test]
        fn area_self() {
            let a = test_area();
            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((-1, -1).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            ));
        }

        #[test]
        fn area_with_a_1x1_overlap() {
            let a = test_area();
            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((-2, -2).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((0, -2).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            ));
            debug_assert!(a.intersects(
                AreaBuilder::default()
                    .anchor((-2, 0).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            ));
        }

        #[test]
        fn area_with_only_a_single_shared_edge() {
            let a = test_area();
            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((1, -1).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((1, 1).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((-1, 1).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((-2, 0).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
            debug_assert!(!a.intersects(
                AreaBuilder::default()
                    .anchor((-2, -2).into())
                    .dimensions((1, 1))
                    .build()
                    .unwrap()
            ));
        }
    }
}
