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

// Lightweight data type to represent a point. Should be passed by value.
#[derive(PartialEq, Clone, Copy)]
pub struct Point<U> {
    inner: (U, U),
}

impl<U> std::fmt::Debug for Point<U>
where
    U: num::PrimInt + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl<U> From<(U, U)> for Point<U>
where
    U: num::PrimInt,
{
    fn from(xy: (U, U)) -> Self {
        Point { inner: xy }
    }
}

impl<U> Into<(U, U)> for Point<U>
where
    U: num::PrimInt,
{
    fn into(self) -> (U, U) {
        self.inner
    }
}

impl<U> std::ops::Add for Point<U>
where
    U: num::PrimInt,
{
    type Output = Point<U>;
    fn add(self, other: Point<U>) -> Point<U> {
        let new_x: U = self.x() + other.x();
        let new_y: U = self.y() + other.y();
        Point {
            inner: (new_x, new_y),
        }
    }
}

impl<U> std::ops::Sub for Point<U>
where
    U: num::PrimInt,
{
    type Output = Point<U>;
    fn sub(self, other: Point<U>) -> Point<U> {
        let new_x: U = self.x() - other.x();
        let new_y: U = self.y() - other.y();
        Point {
            inner: (new_x, new_y),
        }
    }
}

impl<U> Point<U>
where
    U: num::PrimInt,
{
    // Accessors //
    pub fn x(&self) -> U {
        self.inner.0
    }

    pub fn y(&self) -> U {
        self.inner.1
    }

    // XY::dir_towards() expects that:
    //   - (0, 0) is in the top-left corner,
    //   - +x is right (east),
    //   - -y is down (south).
    //
    // Returns 0 if east or northeast of self,
    //         1 if north or northwest of self,
    //         2 if south or southeast of self,
    //      or 3 if west or southwest of self.
    pub fn dir_towards(&self, other: Point<U>) -> usize {
        if other.x() > self.x() && other.y() <= self.y() {
            0 // east, northeast
        } else if other.x() <= self.x() && other.y() < self.y() {
            1 // north, northwest
        } else if other.x() < self.x() && other.y() >= self.y() {
            3 // west, southwest
        } else {
            2 // south, southeast
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Point;

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

// XY::dir_towards() expects that:
//   - (0, 0) is in the top-left corner,
//   - +x is right (east),
//   - -y is down (south).
//
// This test suite ensures that the following segmentation of the grid around a given point is true
// in all four [plane quadrants](https://en.wikipedia.org/wiki/Quadrant_(plane_geometry)):
//
//          N
//          |
//          |                      |
//       11 1 00                   |
//       11 1 00              III  |  IV
//                                 |
//  W ---33 P 00--> E,x    --------+-------->x
//                                 |
//       33 2 22               II  |  I
//       33 2 22                   |
//          |                      |
//          v                      v
//          S,y                    y
//
#[cfg(test)]
mod quadrant_tests {
    use super::Point;

    #[test]
    fn dir_towards_in_quadrant_i() {
        let origin: Point<i8> = (2, 2).into();

        debug_assert_eq!(origin.dir_towards((2, 1).into()), 1); // Due north
        debug_assert_eq!(origin.dir_towards((3, 1).into()), 0); // Northeast
        debug_assert_eq!(origin.dir_towards((3, 2).into()), 0); // Due east
        debug_assert_eq!(origin.dir_towards((3, 3).into()), 2); // Southeast
        debug_assert_eq!(origin.dir_towards((2, 3).into()), 2); // Due south
        debug_assert_eq!(origin.dir_towards((1, 3).into()), 3); // Southwest
        debug_assert_eq!(origin.dir_towards((1, 2).into()), 3); // Due west
        debug_assert_eq!(origin.dir_towards((1, 1).into()), 1); // Northwest
    }

    #[test]
    fn dir_towards_in_quadrant_ii() {
        let origin: Point<i8> = (-2, 2).into();

        debug_assert_eq!(origin.dir_towards((-2, 1).into()), 1); // Due north
        debug_assert_eq!(origin.dir_towards((-1, 1).into()), 0); // Northeast
        debug_assert_eq!(origin.dir_towards((-1, 2).into()), 0); // Due east
        debug_assert_eq!(origin.dir_towards((-1, 3).into()), 2); // Southeast
        debug_assert_eq!(origin.dir_towards((-2, 3).into()), 2); // Due south
        debug_assert_eq!(origin.dir_towards((-3, 3).into()), 3); // Southwest
        debug_assert_eq!(origin.dir_towards((-3, 2).into()), 3); // Due west
        debug_assert_eq!(origin.dir_towards((-3, 1).into()), 1); // Northwest
    }

    #[test]
    fn dir_towards_in_quadrant_iii() {
        let origin: Point<i8> = (-2, -2).into();

        debug_assert_eq!(origin.dir_towards((-2, -3).into()), 1); // Due north
        debug_assert_eq!(origin.dir_towards((-1, -3).into()), 0); // Northeast
        debug_assert_eq!(origin.dir_towards((-1, -2).into()), 0); // Due east
        debug_assert_eq!(origin.dir_towards((-1, -1).into()), 2); // Southeast
        debug_assert_eq!(origin.dir_towards((-2, -1).into()), 2); // Due south
        debug_assert_eq!(origin.dir_towards((-3, -1).into()), 3); // Southwest
        debug_assert_eq!(origin.dir_towards((-3, -2).into()), 3); // Due west
        debug_assert_eq!(origin.dir_towards((-3, -3).into()), 1); // Northwest
    }

    #[test]
    fn dir_towards_in_quadrant_iv() {
        let origin: Point<i8> = (2, -2).into();

        debug_assert_eq!(origin.dir_towards((2, -3).into()), 1); // Due north
        debug_assert_eq!(origin.dir_towards((3, -3).into()), 0); // Northeast
        debug_assert_eq!(origin.dir_towards((3, -2).into()), 0); // Due east
        debug_assert_eq!(origin.dir_towards((3, -1).into()), 2); // Southeast
        debug_assert_eq!(origin.dir_towards((2, -1).into()), 2); // Due south
        debug_assert_eq!(origin.dir_towards((1, -1).into()), 3); // Southwest
        debug_assert_eq!(origin.dir_towards((1, -2).into()), 3); // Due west
        debug_assert_eq!(origin.dir_towards((1, -3).into()), 1); // Northwest
    }

    #[test]
    fn dir_towards_from_origin() {
        let origin: Point<i8> = (0, 0).into();

        debug_assert_eq!(origin.dir_towards((0, -1).into()), 1); // Due north
        debug_assert_eq!(origin.dir_towards((1, -1).into()), 0); // Northeast
        debug_assert_eq!(origin.dir_towards((1, 0).into()), 0); // Due east
        debug_assert_eq!(origin.dir_towards((1, 1).into()), 2); // Southeast
        debug_assert_eq!(origin.dir_towards((0, 1).into()), 2); // Due south
        debug_assert_eq!(origin.dir_towards((-1, 1).into()), 3); // Southwest
        debug_assert_eq!(origin.dir_towards((-1, 0).into()), 3); // Due west
        debug_assert_eq!(origin.dir_towards((-1, -1).into()), 1); // Northwest
    }
}
