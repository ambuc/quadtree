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

use crate::geometry::point::{Point, PointType};

// Transparent alias. In docs and user-facing APIs, this resolves to ((U, U), (U, U)).
pub type AreaType<U> = (PointType<U>, (U, U));

// Lightweight data type to represent a region.
// The top-left anchor may be positive or negative in either coordinate.
// Defined by a top-left anchor and a width/height.
// The width/height must both be positive and nonzero.
// Should be passed by value.
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct Area<U> {
    inner: AreaType<U>,
}

impl<U> std::fmt::Debug for Area<U>
where
    U: num::PrimInt + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "({:?})->{:?}x{:?}",
            self.anchor(),
            self.width(),
            self.height()
        )
    }
}

impl<U> From<AreaType<U>> for Area<U>
where
    U: num::PrimInt,
{
    fn from((xy, (w, h)): AreaType<U>) -> Self {
        assert!(!w.is_zero());
        assert!(!h.is_zero());
        // Regions shouldn't be negative in dimenision. I guess there's a way to handle the math by
        // going in the (-x,-y) direction, but it seems better to communicate that they shouldn't
        // be.
        assert!(w > U::zero());
        assert!(h > U::zero());
        Area {
            inner: (xy, (w, h)),
        }
    }
}

impl<U> From<&AreaType<U>> for Area<U>
where
    U: num::PrimInt,
{
    fn from((xy, (w, h)): &AreaType<U>) -> Self {
        assert!(!w.is_zero());
        assert!(!h.is_zero());
        // Regions shouldn't be negative in dimenision. I guess there's a way to handle the math by
        // going in the (-x,-y) direction, but it seems better to communicate that they shouldn't
        // be.
        assert!(*w > U::zero());
        assert!(*h > U::zero());
        Area {
            inner: (*xy, (*w, *h)),
        }
    }
}

impl<U> Into<AreaType<U>> for Area<U> {
    fn into(self) -> AreaType<U> {
        self.inner
    }
}

impl<U> Area<U>
where
    U: num::PrimInt,
{
    // Accessors
    pub fn inner(&self) -> &AreaType<U> {
        &self.inner
    }

    pub fn anchor(&self) -> Point<U> {
        self.inner.0.into()
    }
    pub fn dimensions(&self) -> PointType<U> {
        self.inner.1
    }
    // Properties
    // // Measurements
    pub fn width(&self) -> U {
        self.dimensions().0
    }
    pub fn height(&self) -> U {
        self.dimensions().1
    }
    // // Positions
    fn top(&self) -> U {
        self.anchor().y()
    }
    fn bottom(&self) -> U {
        self.anchor().y() + self.height()
    }
    fn left(&self) -> U {
        self.anchor().x()
    }
    fn right(&self) -> U {
        self.anchor().x() + self.width()
    }
    // // Coordinates
    #[allow(dead_code)]
    pub fn tl_pt(&self) -> Point<U> {
        (self.left(), self.top()).into()
    }
    #[allow(dead_code)]
    pub fn tr_pt(&self) -> Point<U> {
        (self.right(), self.top()).into()
    }
    #[allow(dead_code)]
    pub fn bl_pt(&self) -> Point<U> {
        (self.left(), self.bottom()).into()
    }
    #[allow(dead_code)]
    pub fn br_pt(&self) -> Point<U> {
        (self.right(), self.bottom()).into()
    }
    // Evaluation

    // Whether or not an area intersects another area.
    pub fn intersects(self, other: Area<U>) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }
    // Whether or not an area wholly contains another area.
    pub fn contains(self, other: Area<U>) -> bool {
        other.right() <= self.right()
            && other.left() >= self.left()
            && other.top() >= self.top()
            && other.bottom() <= self.bottom()
    }

    // Whether or not an area contains a point.
    //
    // This only gets used in tests at the moment.
    #[allow(dead_code)]
    pub fn contains_pt(self, pt: Point<U>) -> bool {
        self.contains((pt.into(), /*default dimensions*/ (U::one(), U::one())).into())
    }
}

#[cfg(test)]
mod tests {
    use super::Area;

    mod invalid_area_creation {
        use super::*;

        #[test]
        #[should_panic]
        fn negative_width() {
            let _a: Area<i8> = ((0, 0), (-1, 4)).into();
        }

        #[test]
        #[should_panic]
        fn negative_height() {
            let _a: Area<i8> = ((0, 0), (1, -4)).into();
        }

        #[test]
        #[should_panic]
        fn zero_width() {
            let _a: Area<i8> = ((0, 0), (0, 4)).into();
        }

        #[test]
        #[should_panic]
        fn zero_height() {
            let _a: Area<i8> = ((0, 0), (1, 0)).into();
        }
    }

    mod creation_in_quadrant {
        use super::*;

        #[test]
        fn i() {
            let _a: Area<i8> = ((1, 1), (1, 1)).into();
        }

        #[test]
        fn ii() {
            let _a: Area<i8> = ((-1, 1), (1, 1)).into();
        }

        #[test]
        fn iii() {
            let _a: Area<i8> = ((-1, -1), (1, 1)).into();
        }

        #[test]
        fn iv() {
            let _a: Area<i8> = ((1, -1), (1, 1)).into();
        }
    }

    mod properties {
        use super::*;
        fn mk() -> Area<i8> {
            ((3, 4), (5, 7)).into()
        }
        #[test]
        fn properties() {
            let a = mk();
            debug_assert_eq!(a.anchor(), (3, 4).into());
            debug_assert_eq!(a.dimensions(), (5, 7));
            debug_assert_eq!(a.width(), 5);
            debug_assert_eq!(a.height(), 7);

            debug_assert_eq!(a.left(), 3);
            debug_assert_eq!(a.top(), 4);
            debug_assert_eq!(a.right(), /*3+5*/ 8);
            debug_assert_eq!(a.bottom(), /*4+7*/ 11);

            debug_assert_eq!(a.tl_pt(), (3, 4).into());
            debug_assert_eq!(a.tr_pt(), (8, 4).into());
            debug_assert_eq!(a.bl_pt(), (3, 11).into());
            debug_assert_eq!(a.br_pt(), (8, 11).into());
        }
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

            debug_assert!(a.contains(((1, 1), (1, 1)).into()));
            debug_assert!(a.contains(((1, 2), (1, 1)).into()));
            debug_assert!(a.contains(((2, 1), (1, 1)).into()));
            debug_assert!(a.contains(((2, 2), (1, 1)).into()));
        }

        #[test]
        fn contains_self() {
            let a = test_area();

            debug_assert!(a.contains(((1, 1), (2, 2)).into()));
        }

        #[test]
        fn no_neighboring_1x1s() {
            let a = test_area();

            debug_assert!(!a.contains(((0, 0), (1, 1)).into()));
            debug_assert!(!a.contains(((1, 0), (1, 1)).into()));
            debug_assert!(!a.contains(((2, 0), (1, 1)).into()));
            debug_assert!(!a.contains(((3, 0), (1, 1)).into()));
            debug_assert!(!a.contains(((4, 0), (1, 1)).into()));
            debug_assert!(!a.contains(((0, 3), (1, 1)).into()));
            debug_assert!(!a.contains(((1, 3), (1, 1)).into()));
            debug_assert!(!a.contains(((2, 3), (1, 1)).into()));
            debug_assert!(!a.contains(((3, 3), (1, 1)).into()));
            debug_assert!(!a.contains(((4, 3), (1, 1)).into()));
            debug_assert!(!a.contains(((0, 1), (1, 1)).into()));
            debug_assert!(!a.contains(((0, 2), (1, 1)).into()));
            debug_assert!(!a.contains(((0, 3), (1, 1)).into()));
            debug_assert!(!a.contains(((3, 1), (1, 1)).into()));
            debug_assert!(!a.contains(((3, 2), (1, 1)).into()));
            debug_assert!(!a.contains(((3, 3), (1, 1)).into()));
        }

        #[test]
        fn no_overlapping_2x2s() {
            let a = test_area();

            debug_assert!(!a.contains(((0, 0), (2, 2)).into()));
            debug_assert!(!a.contains(((2, 2), (2, 2)).into()));
        }

        #[test]
        fn no_overlapping_3x3s() {
            let a = test_area();

            debug_assert!(!a.contains(((0, 0), (3, 3)).into()));
            debug_assert!(!a.contains(((1, 0), (3, 3)).into()));
            debug_assert!(!a.contains(((1, 1), (3, 3)).into()));
            debug_assert!(!a.contains(((1, 1), (3, 3)).into()));
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
            ((-1, -1), (2, 2)).into()
        }

        #[test]
        fn contains_one() {
            let a = test_area();

            debug_assert!(a.contains(((-1, -1), (1, 1)).into()));
            debug_assert!(a.contains(((0, -1), (1, 1)).into()));
            debug_assert!(a.contains(((0, 0), (1, 1)).into()));
            debug_assert!(a.contains(((-1, 0), (1, 1)).into()));
        }

        #[test]
        fn contains_self() {
            let a = test_area();

            debug_assert!(a.contains(((-1, -1), (2, 2)).into()));
        }

        #[test]
        fn no_neighboring_1x1s() {
            let a = test_area();

            debug_assert!(!a.contains(((-2, -2), (1, 1)).into()));
            debug_assert!(!a.contains(((-2, -1), (1, 1)).into()));
            debug_assert!(!a.contains(((-2, 0), (1, 1)).into()));
            debug_assert!(!a.contains(((-2, 1), (1, 1)).into()));
            debug_assert!(!a.contains(((-2, 2), (1, 1)).into()));
            debug_assert!(!a.contains(((-1, 2), (1, 1)).into()));
            debug_assert!(!a.contains(((0, 2), (1, 1)).into()));
            debug_assert!(!a.contains(((1, 2), (1, 1)).into()));
            debug_assert!(!a.contains(((2, 2), (1, 1)).into()));
            debug_assert!(!a.contains(((2, 1), (1, 1)).into()));
            debug_assert!(!a.contains(((2, 0), (1, 1)).into()));
            debug_assert!(!a.contains(((2, -1), (1, 1)).into()));
            debug_assert!(!a.contains(((2, -2), (1, 1)).into()));
            debug_assert!(!a.contains(((1, -2), (1, 1)).into()));
            debug_assert!(!a.contains(((0, -2), (1, 1)).into()));
            debug_assert!(!a.contains(((-1, -2), (1, 1)).into()));
        }

        #[test]
        fn no_overlapping_2x2s() {
            let a = test_area();

            debug_assert!(!a.contains(((0, 0), (2, 2)).into()));
            debug_assert!(!a.contains(((2, 2), (2, 2)).into()));
            debug_assert!(!a.contains(((-2, -2), (2, 2)).into()));
        }

        #[test]
        fn no_overlapping_3x3s() {
            let a = test_area();

            debug_assert!(!a.contains(((0, 0), (3, 3)).into()));
            debug_assert!(!a.contains(((1, 0), (3, 3)).into()));
            debug_assert!(!a.contains(((-1, -1), (3, 3)).into()));
            debug_assert!(!a.contains(((-1, 1), (3, 3)).into()));
            debug_assert!(!a.contains(((-2, 1), (3, 3)).into()));
            debug_assert!(!a.contains(((-2, -2), (3, 3)).into()));
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
            ((2, 2), (2, 2)).into()
        }

        // All the 1x1s obviously contains.
        #[test]
        fn area_1x1() {
            let a = test_area();

            debug_assert!(a.intersects(((2, 2), (1, 1)).into()));
            debug_assert!(a.intersects(((2, 3), (1, 1)).into()));
            debug_assert!(a.intersects(((3, 2), (1, 1)).into()));
            debug_assert!(a.intersects(((3, 3), (1, 1)).into()));
        }

        // And the one 2x2 obviously contained.
        #[test]
        fn area_2x2() {
            let a = test_area();

            debug_assert!(a.intersects(((2, 2), (2, 2)).into()));
        }

        // But a single edge shared is not enough.
        #[test]
        fn area_with_only_a_single_shared_edge() {
            let a = test_area();

            debug_assert!(!a.intersects(((1, 1), (1, 1)).into()));
            debug_assert!(!a.intersects(((1, 1), (2, 1)).into()));
            debug_assert!(!a.intersects(((1, 1), (4, 1)).into()));
            debug_assert!(!a.intersects(((2, 1), (1, 1)).into()));
            debug_assert!(!a.intersects(((3, 1), (2, 1)).into()));
            debug_assert!(!a.intersects(((4, 1), (2, 1)).into()));
            debug_assert!(!a.intersects(((1, 1), (1, 2)).into()));
            debug_assert!(!a.intersects(((1, 2), (1, 2)).into()));
            debug_assert!(!a.intersects(((1, 3), (1, 2)).into()));
            debug_assert!(!a.intersects(((1, 4), (1, 2)).into()));
            debug_assert!(!a.intersects(((2, 4), (1, 1)).into()));
            debug_assert!(!a.intersects(((3, 4), (1, 1)).into()));
            debug_assert!(!a.intersects(((4, 4), (1, 1)).into()));
        }

        // But intersecting a 1x1 region counts.
        #[test]
        fn area_with_a_1x1_overlap() {
            let a = test_area();

            debug_assert!(a.intersects(((1, 1), (2, 2)).into()));
            debug_assert!(a.intersects(((0, 0), (3, 3)).into()));
            debug_assert!(a.intersects(((3, 3), (2, 2)).into()));
            debug_assert!(a.intersects(((1, 3), (2, 2)).into()));
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
            debug_assert!(a.intersects(((-1, -1), (1, 1)).into()));
            debug_assert!(a.intersects(((-1, 0), (1, 1)).into()));
            debug_assert!(a.intersects(((0, 0), (1, 1)).into()));
            debug_assert!(a.intersects(((0, -1), (1, 1)).into()));
        }

        #[test]
        fn area_self() {
            let a = test_area();
            debug_assert!(a.intersects(((-1, -1), (2, 2)).into()));
        }

        #[test]
        fn area_with_a_1x1_overlap() {
            let a = test_area();
            debug_assert!(a.intersects(((-2, -2), (2, 2)).into()));
            debug_assert!(a.intersects(((0, -2), (2, 2)).into()));
            debug_assert!(a.intersects(((0, 0), (2, 2)).into()));
            debug_assert!(a.intersects(((-2, 0), (2, 2)).into()));
        }

        #[test]
        fn area_with_only_a_single_shared_edge() {
            let a = test_area();
            debug_assert!(!a.intersects(((1, -1), (1, 1)).into()));
            debug_assert!(!a.intersects(((1, 1), (1, 1)).into()));
            debug_assert!(!a.intersects(((-1, 1), (1, 1)).into()));
            debug_assert!(!a.intersects(((-2, 0), (1, 1)).into()));
            debug_assert!(!a.intersects(((-2, -2), (1, 1)).into()));
        }
    }
}
