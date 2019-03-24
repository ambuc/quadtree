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

mod point_tests {
    use quadtree_rs::point::Point;

    #[test]
    fn builder() {
        let p: Point<i8> = (1, 2).into();
        debug_assert_eq!(p.x(), 1);
        debug_assert_eq!(p.y(), 2);
    }

    #[test]
    fn xy_addition() {
        debug_assert_eq!(Point::from((0, 0)) + Point::from((0, 1)), (0, 1).into());
        debug_assert_eq!(Point::from((0, 1)) + Point::from((0, 1)), (0, 2).into());
        debug_assert_eq!(Point::from((1, 1)) + Point::from((0, 0)), (1, 1).into());
        debug_assert_eq!(Point::from((1, 0)) + Point::from((0, 1)), (1, 1).into());
        debug_assert_eq!(Point::from((0, 0)) + Point::from((4, 5)), (4, 5).into());
        debug_assert_eq!(Point::from((4, 5)) + Point::from((0, 0)), (4, 5).into());
    }

    #[test]
    fn xy_subtraction() {
        debug_assert_eq!(Point::from((0, 1)) - Point::from((0, 0)), (0, 1).into());
        debug_assert_eq!(Point::from((0, 1)) - Point::from((0, 1)), (0, 0).into());
        debug_assert_eq!(Point::from((1, 1)) - Point::from((0, 0)), (1, 1).into());
        debug_assert_eq!(Point::from((1, 1)) - Point::from((0, 1)), (1, 0).into());
        debug_assert_eq!(Point::from((4, 5)) - Point::from((2, 2)), (2, 3).into());
        debug_assert_eq!(Point::from((4, 5)) - Point::from((0, 0)), (4, 5).into());
    }

    // Test addition / subtraction which reaches into the realm of negative numbers.

    #[test]
    fn subtracting_positive_numbers() {
        debug_assert_eq!(Point::from((0, 0)) - (1, 1).into(), (-1, -1).into());
        debug_assert_eq!(Point::from((0, 0)) - (0, 1).into(), (0, -1).into());
        debug_assert_eq!(Point::from((0, 0)) - (1, 0).into(), (-1, 0).into());

        debug_assert_eq!(Point::from((1, 10)) - (2, 20).into(), (-1, -10).into());
    }

    #[test]
    fn adding_negative_numbers() {
        debug_assert_eq!(Point::from((0, 0)) + (-1, 0).into(), (-1, 0).into());
        debug_assert_eq!(Point::from((0, 0)) + (-1, -1).into(), (-1, -1).into());
        debug_assert_eq!(Point::from((0, 0)) + (0, -1).into(), (0, -1).into());

        debug_assert_eq!(Point::from((1, 10)) + (-2, -20).into(), (-1, -10).into());
    }
}
