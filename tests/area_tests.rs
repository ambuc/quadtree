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
    use quadtree_rs::geometry::Area;

    mod builder {
        use super::*;

        #[test]
        fn builder() {
            let a: Area<i8> = ((0, 0), (2, 2)).into();
            debug_assert_eq!(a.width(), 2);
        }
    }

    #[test]
    #[should_panic]
    fn bad_dims() {
        for (h, w) in [(-1, 4), (1, -4), (0, 4), (1, 0)] {
            Area::new(h, w);
        }
    }

    #[test]
    fn point_in_all_quadrants() {
        for p in [(1, 1), (-1, 1), (1, -1), (-1, -1)] {
            let _a: Area<i8> = p.into();
        }
    }

    #[test]
    fn properties() {
        let a: Area<u8> = ((3, 4), (5, 7)).into();

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
            ((1, 1), (2, 2)).into()
        }

        #[test]
        fn all_component_1x1s() {
            let a = test_area();

            debug_assert!(a.contains(((1, 1), (1, 1))));
            debug_assert!(a.contains(((1, 2), (1, 1))));
            debug_assert!(a.contains(((2, 1), (1, 1))));
            debug_assert!(a.contains(((2, 2), (1, 1))));
        }

        #[test]
        fn contains_self() {
            let a = test_area();

            debug_assert!(a.contains(((1, 1), (2, 2))));
        }

        #[test]
        fn no_neighboring_1x1s() {
            let a = test_area();

            debug_assert!(!a.contains(((0, 0), (1, 1))));
            debug_assert!(!a.contains(((1, 0), (1, 1))));
            debug_assert!(!a.contains(((2, 0), (1, 1))));
            debug_assert!(!a.contains(((3, 0), (1, 1))));
            debug_assert!(!a.contains(((4, 0), (1, 1))));
            debug_assert!(!a.contains(((0, 3), (1, 1))));
            debug_assert!(!a.contains(((1, 3), (1, 1))));
            debug_assert!(!a.contains(((2, 3), (1, 1))));
            debug_assert!(!a.contains(((3, 3), (1, 1))));
            debug_assert!(!a.contains(((4, 3), (1, 1))));
            debug_assert!(!a.contains(((0, 1), (1, 1))));
            debug_assert!(!a.contains(((0, 2), (1, 1))));
            debug_assert!(!a.contains(((0, 3), (1, 1))));
            debug_assert!(!a.contains(((3, 1), (1, 1))));
            debug_assert!(!a.contains(((3, 2), (1, 1))));
            debug_assert!(!a.contains(((3, 3), (1, 1))));
        }

        #[test]
        fn no_overlapping_2x2s() {
            let a = test_area();

            debug_assert!(!a.contains(((0, 0), (2, 2))));
            debug_assert!(!a.contains(((2, 2), (2, 2))));
        }

        #[test]
        fn no_overlapping_3x3s() {
            let a = test_area();

            debug_assert!(!a.contains(((0, 0), (3, 3))));
            debug_assert!(!a.contains(((1, 0), (3, 3))));
            debug_assert!(!a.contains(((1, 1), (3, 3))));
            debug_assert!(!a.contains(((1, 1), (3, 3))));
        }

        #[test]
        fn contains_pt() {
            let a = test_area();

            // DOES contain:
            debug_assert!(a.contains_pt((1, 1)));
            debug_assert!(a.contains_pt((1, 2)));
            debug_assert!(a.contains_pt((2, 1)));
            debug_assert!(a.contains_pt((2, 2)));

            // Does NOT contain:
            debug_assert!(!a.contains_pt((0, 0)));
            debug_assert!(!a.contains_pt((0, 1)));
            debug_assert!(!a.contains_pt((0, 2)));
            debug_assert!(!a.contains_pt((0, 3)));
            debug_assert!(!a.contains_pt((1, 0)));
            debug_assert!(!a.contains_pt((2, 0)));
            debug_assert!(!a.contains_pt((3, 0)));
            debug_assert!(!a.contains_pt((3, 0)));
            debug_assert!(!a.contains_pt((3, 1)));
            debug_assert!(!a.contains_pt((3, 2)));
            debug_assert!(!a.contains_pt((3, 3)));
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
            ((-1, -1), (2, 2)).into()
        }

        #[test]
        fn contains_one() {
            let a = test_area();

            debug_assert!(a.contains(((-1, -1), (1, 1))));
            debug_assert!(a.contains(((0, -1), (1, 1))));
            debug_assert!(a.contains(((0, 0), (1, 1))));
            debug_assert!(a.contains(((-1, 0), (1, 1))));
        }

        #[test]
        fn contains_self() {
            let a = test_area();

            debug_assert!(a.contains(((-1, -1), (2, 2))));
        }

        #[test]
        fn no_neighboring_1x1s() {
            let a = test_area();

            debug_assert!(!a.contains(((-2, -2), (1, 1))));
            debug_assert!(!a.contains(((-2, -1), (1, 1))));
            debug_assert!(!a.contains(((-2, 0), (1, 1))));
            debug_assert!(!a.contains(((-2, 1), (1, 1))));
            debug_assert!(!a.contains(((-2, 2), (1, 1))));
            debug_assert!(!a.contains(((-1, 2), (1, 1))));
            debug_assert!(!a.contains(((0, 2), (1, 1))));
            debug_assert!(!a.contains(((1, 2), (1, 1))));
            debug_assert!(!a.contains(((2, 2), (1, 1))));
            debug_assert!(!a.contains(((2, 1), (1, 1))));
            debug_assert!(!a.contains(((2, 0), (1, 1))));
            debug_assert!(!a.contains(((2, -1), (1, 1))));
            debug_assert!(!a.contains(((2, -2), (1, 1))));
            debug_assert!(!a.contains(((1, -2), (1, 1))));
            debug_assert!(!a.contains(((0, -2), (1, 1))));
            debug_assert!(!a.contains(((-1, -2), (1, 1))));
        }

        #[test]
        fn no_overlapping_2x2s() {
            let a = test_area();

            debug_assert!(!a.contains(((0, 0), (2, 2))));
            debug_assert!(!a.contains(((2, 2), (2, 2))));
            debug_assert!(!a.contains(((-2, -2), (2, 2))));
        }

        #[test]
        fn no_overlapping_3x3s() {
            let a = test_area();

            debug_assert!(!a.contains(((0, 0), (3, 3))));
            debug_assert!(!a.contains(((1, 0), (3, 3))));
            debug_assert!(!a.contains(((-1, -1), (3, 3))));
            debug_assert!(!a.contains(((-1, 1), (3, 3))));
            debug_assert!(!a.contains(((-2, 1), (3, 3))));
            debug_assert!(!a.contains(((-2, -2), (3, 3))));
        }

        #[test]
        fn contains_pt() {
            let a = test_area();

            // DOES contain:
            debug_assert!(a.contains_pt((-1, -1)));
            debug_assert!(a.contains_pt((-1, 0)));
            debug_assert!(a.contains_pt((0, -1)));
            debug_assert!(a.contains_pt((0, 0)));

            // Does NOT contain:
            debug_assert!(!a.contains_pt((-2, -2)));
            debug_assert!(!a.contains_pt((-2, -1)));
            debug_assert!(!a.contains_pt((-2, 0)));
            debug_assert!(!a.contains_pt((-2, 1)));
            debug_assert!(!a.contains_pt((-2, 2)));
            debug_assert!(!a.contains_pt((-1, 2)));
            debug_assert!(!a.contains_pt((0, 2)));
            debug_assert!(!a.contains_pt((1, 2)));
            debug_assert!(!a.contains_pt((2, 2)));
            debug_assert!(!a.contains_pt((2, 1)));
            debug_assert!(!a.contains_pt((2, 0)));
            debug_assert!(!a.contains_pt((2, -1)));
            debug_assert!(!a.contains_pt((2, -2)));
            debug_assert!(!a.contains_pt((1, -2)));
            debug_assert!(!a.contains_pt((0, -2)));
            debug_assert!(!a.contains_pt((-1, -2)));
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
            ((2, 2), (2, 2)).into()
        }

        // All the 1x1s obviously contains.
        #[test]
        fn area_1x1() {
            let a = test_area();

            debug_assert!(a.intersects(((2, 2), (1, 1))));
            debug_assert!(a.intersects(((2, 3), (1, 1))));
            debug_assert!(a.intersects(((3, 2), (1, 1))));
            debug_assert!(a.intersects(((3, 3), (1, 1))));
        }

        // And the one 2x2 obviously contained.
        #[test]
        fn area_2x2() {
            let a = test_area();

            debug_assert!(a.intersects(((2, 2), (2, 2))));
        }

        // But a single edge shared is not enough.
        #[test]
        fn area_with_only_a_single_shared_edge() {
            let a = test_area();

            debug_assert!(!a.intersects(((1, 1), (1, 1))));
            debug_assert!(!a.intersects(((1, 1), (2, 1))));
            debug_assert!(!a.intersects(((1, 1), (4, 1))));
            debug_assert!(!a.intersects(((2, 1), (1, 1))));
            debug_assert!(!a.intersects(((3, 1), (2, 1))));
            debug_assert!(!a.intersects(((4, 1), (2, 1))));
            debug_assert!(!a.intersects(((1, 1), (1, 2))));
            debug_assert!(!a.intersects(((1, 2), (1, 2))));
            debug_assert!(!a.intersects(((1, 3), (1, 2))));
            debug_assert!(!a.intersects(((1, 4), (1, 2))));
            debug_assert!(!a.intersects(((2, 4), (1, 1))));
            debug_assert!(!a.intersects(((3, 4), (1, 1))));
            debug_assert!(!a.intersects(((4, 4), (1, 1))));
        }

        // But intersecting a 1x1 region counts.
        #[test]
        fn area_with_a_1x1_overlap() {
            let a = test_area();

            debug_assert!(a.intersects(((1, 1), (2, 2))));
            debug_assert!(a.intersects(((0, 0), (3, 3))));
            debug_assert!(a.intersects(((3, 3), (2, 2))));
            debug_assert!(a.intersects(((1, 3), (2, 2))));
        }

        #[test]
        fn regression_test() {
            let a: Area<u8> = ((3, 3), (2, 2)).into();
            let b: Area<u8> = ((0, 0), (6, 6)).into();

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
            ((-1, -1), (2, 2)).into()
        }

        #[test]
        fn area_1x1() {
            let a = test_area();
            debug_assert!(a.intersects(((-1, -1), (1, 1))));
            debug_assert!(a.intersects(((-1, 0), (1, 1))));
            debug_assert!(a.intersects(((0, 0), (1, 1))));
            debug_assert!(a.intersects(((0, -1), (1, 1))));
        }

        #[test]
        fn area_self() {
            let a = test_area();
            debug_assert!(a.intersects(((-1, -1), (2, 2))));
        }

        #[test]
        fn area_with_a_1x1_overlap() {
            let a = test_area();
            debug_assert!(a.intersects(((-2, -2), (2, 2))));
            debug_assert!(a.intersects(((0, -2), (2, 2))));
            debug_assert!(a.intersects(((0, 0), (2, 2))));
            debug_assert!(a.intersects(((-2, 0), (2, 2))));
        }

        #[test]
        fn area_with_only_a_single_shared_edge() {
            let a = test_area();
            debug_assert!(!a.intersects(((1, -1), (1, 1))));
            debug_assert!(!a.intersects(((1, 1), (1, 1))));
            debug_assert!(!a.intersects(((-1, 1), (1, 1))));
            debug_assert!(!a.intersects(((-2, 0), (1, 1))));
            debug_assert!(!a.intersects(((-2, -2), (1, 1))));
        }
    }
}
