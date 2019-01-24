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

use crate::point::Point;
use num_traits::sign;

pub type AreaType<U> = ((U, U), (U, U));

// Lightweight data type to represent a region.
// Defined by a top-left anchor and a width/height.
// Should be passed by value.
//
// TODO(ambuc): Should this be parameterized across sign::Signed as well? Write unit tests for this
// case.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Area<U> {
    inner: AreaType<U>,
}

impl<U> std::fmt::Debug for Area<U>
where
    U: num::PrimInt + sign::Unsigned + std::fmt::Debug,
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

impl<U> Area<U>
where
    U: num::PrimInt + sign::Unsigned,
{
    pub fn anchor(&self) -> Point<U> {
        self.inner.0.into()
    }

    pub fn width(&self) -> U {
        self.dimensions().0
    }

    pub fn height(&self) -> U {
        self.dimensions().1
    }

    pub fn inner(&self) -> &AreaType<U> {
        &self.inner
    }

    fn dimensions(&self) -> (U, U) {
        self.inner.1
    }
}

impl<U> From<AreaType<U>> for Area<U>
where
    U: num::PrimInt + sign::Unsigned,
{
    fn from((xy, (w, h)): AreaType<U>) -> Self {
        assert!(!w.is_zero());
        assert!(!h.is_zero());
        Area {
            inner: (xy, (w, h)),
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
    U: num::PrimInt + sign::Unsigned,
{
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
        self.contains((pt.into(), (num::One::one(), num::One::one())).into())
    }

    // Whether or not an area intersects another area.
    pub fn intersects(self, other: Area<U>) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }

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
}

#[cfg(test)]
mod tests {
    use super::Area;
    #[test]
    fn test_area_contains() {
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

        let a: Area<u8> = ((1, 1), (2, 2)).into();

        // Does contain all component 1x1s
        debug_assert!(a.contains(((1, 1), (1, 1)).into()));
        debug_assert!(a.contains(((1, 2), (1, 1)).into()));
        debug_assert!(a.contains(((2, 1), (1, 1)).into()));
        debug_assert!(a.contains(((2, 2), (1, 1)).into()));

        // Does contain self
        debug_assert!(a.contains(((1, 1), (2, 2)).into()));

        // Does NOT contain all neighboring 1x1s
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

        // Does NOT contain overlapping 2x2s
        debug_assert!(!a.contains(((0, 0), (2, 2)).into()));
        debug_assert!(!a.contains(((2, 2), (2, 2)).into()));

        // Does NOT contain overlapping 3x3s
        debug_assert!(!a.contains(((0, 0), (3, 3)).into()));
        debug_assert!(!a.contains(((1, 0), (3, 3)).into()));
        debug_assert!(!a.contains(((1, 1), (3, 3)).into()));
        debug_assert!(!a.contains(((1, 1), (3, 3)).into()));
    }

    #[test]
    fn test_area_contains_pt() {
        let a: Area<u8> = ((1, 1), (2, 2)).into();

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
    #[test]
    fn area_intersects_area() {
        let a: Area<u8> = ((2, 2), (2, 2)).into();

        // All the 1x1s obviously contains
        debug_assert!(a.intersects(((2, 2), (1, 1)).into()));
        debug_assert!(a.intersects(((2, 3), (1, 1)).into()));
        debug_assert!(a.intersects(((3, 2), (1, 1)).into()));
        debug_assert!(a.intersects(((3, 3), (1, 1)).into()));

        // And the one 2x2 obviously contained
        debug_assert!(a.intersects(((2, 2), (2, 2)).into()));

        // But a single edge shared is not enough.
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

        // But intersecting a 1x1 region counts.
        debug_assert!(a.intersects(((1, 1), (2, 2)).into()));
        debug_assert!(a.intersects(((0, 0), (3, 3)).into()));
        debug_assert!(a.intersects(((3, 3), (2, 2)).into()));
        debug_assert!(a.intersects(((1, 3), (2, 2)).into()));
    }

    #[test]
    fn area_intersects_area_regression_test() {
        let a: Area<u8> = ((3, 3), (2, 2)).into();
        let b: Area<u8> = ((0, 0), (6, 6)).into();

        debug_assert!(b.intersects(a));
        debug_assert!(a.intersects(b));
    }
}
