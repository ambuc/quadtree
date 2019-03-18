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

//! A type representing an point in space.

// d8888b.  .d88b.  d888888b d8b   db d888888b
// 88  `8D .8P  Y8.   `88'   888o  88 `~~88~~'
// 88oodD' 88    88    88    88V8o 88    88
// 88~~~   88    88    88    88 V8o88    88
// 88      `8b  d8'   .88.   88  V888    88
// 88       `Y88P'  Y888888P VP   V8P    YP

// Transparent alias. In docs and user-facing APIs, this resolves to (U, U).
pub type PointType<U> = (U, U);

/// A type representing a point in space. Should be passed by value.
#[derive(PartialEq, Eq, Clone, Copy, Hash, Builder)]
pub struct Point<U> {
    x: U,
    y: U,
}

/// foo
impl<U> std::fmt::Debug for Point<U>
where
    U: num::PrimInt + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}x{:?}", self.x, self.y)
    }
}

impl<U> From<PointType<U>> for Point<U>
where
    U: num::PrimInt,
{
    fn from((x, y): PointType<U>) -> Self {
        Point { x, y }
    }
}

impl<U> From<&PointType<U>> for Point<U>
where
    U: num::PrimInt,
{
    fn from((x, y): &PointType<U>) -> Self {
        Point { x: *x, y: *y }
    }
}

impl<U> Into<PointType<U>> for Point<U>
where
    U: num::PrimInt,
{
    fn into(self) -> PointType<U> {
        (self.x, self.y)
    }
}

impl<U> std::ops::Add for Point<U>
where
    U: num::PrimInt,
{
    type Output = Point<U>;
    fn add(self, other: Point<U>) -> Point<U> {
        Point {
            x: self.x() + other.x(),
            y: self.y() + other.y(),
        }
    }
}

impl<U> std::ops::Sub for Point<U>
where
    U: num::PrimInt,
{
    type Output = Point<U>;
    fn sub(self, other: Point<U>) -> Point<U> {
        Point {
            x: self.x() - other.x(),
            y: self.y() - other.y(),
        }
    }
}

impl<U> Point<U>
where
    U: num::PrimInt,
{
    // Accessors //
    pub fn x(&self) -> U {
        self.x
    }

    pub fn y(&self) -> U {
        self.y
    }
}

#[cfg(test)]
mod tests {
    use super::{Point, PointBuilder};

    #[test]
    fn builder() {
        let p: Point<i8> = PointBuilder::default().x(1).y(2).build().unwrap();
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
