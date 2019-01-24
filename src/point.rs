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
#[derive(PartialEq)]
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

    // Returns 0 if northeast of self,
    //         1 if northwest of self,
    //         2 if southeast of self,
    //      or 3 if southwest of self. Will never return 4 or more.
    pub fn dir_towards(&self, other: Point<U>) -> usize {
        if other.y() < self.y() {
            if other.x() < self.x() {
                1 // northwest
            } else {
                0 // northeast
            }
        } else if other.x() < self.x() {
            3 // southwest
        } else {
            2 // southeast
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
}
