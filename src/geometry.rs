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

use num::PrimInt;
#[cfg(feature = "serde")]
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    cmp::PartialOrd,
    default::Default,
    fmt::Debug,
    ops::{
        Add,
        Sub,
    },
};

/// A rectangular region in 2d space.
///
/// Lightweight, should be passed by value. Defined by its top-left anchor, width, and height.
///
/// **NB:**
///   - The top-left anchor can be any valid `(U, U)` coordinate, positive or negative, in any
///   quadrant.
///   - The width and height must both be positive and nonzero.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct Area<U>
where
    U: PrimInt + Default + PartialOrd,
{
    anchor: Point<U>,
    dimensions: (U, U),
}

impl<U> Debug for Area<U>
where
    U: PrimInt + Default + Debug,
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

/// Why this custom From<>? Useful for type coercion:
///
/// ```
/// use quadtree_rs::{area::{Area, AreaBuilder}, point::Point};
///
/// let area: Area<_> = AreaBuilder::default()
///     .anchor(Point{x:1, y:2})
///     .dimensions((3,4))
///     .build().unwrap();
/// let (anchor, dims) = area.into();
/// assert_eq!(anchor, (1,2));
/// assert_eq!(dims, (3,4));
/// ```
impl<U> From<Area<U>> for ((U, U), (U, U))
where
    U: PrimInt + Default,
{
    fn from(value: Area<U>) -> Self {
        (value.anchor.into(), value.dimensions())
    }
}

impl<U> Area<U>
where
    U: PrimInt + Default,
{
    /// Construct a new [`Area`].
    /// # Panics
    /// Panics if either width or height is negative.
    pub fn new(width: U, height: U) -> Self {
        assert!(width > U::zero() && height > U::zero());
        Self {
            anchor: (U::one(), U::one()).into(),
            dimensions: (width, height),
        }
    }

    /// Unit area with width and height of one.
    pub fn unit() -> Self {
        Self::new(U::one(), U::one())
    }

    /// Returns a new area with it's top-left point set to `anchor`
    pub fn at(self, anchor: impl Into<Point<U>>) -> Self {
        let Self {
            anchor: _,
            dimensions,
        } = self;
        Self {
            anchor: anchor.into(),
            dimensions,
        }
    }

    /// The top-left coordinate (anchor) of the region.
    pub fn anchor(&self) -> Point<U> {
        self.anchor
    }

    /// The width of the region.
    pub fn width(&self) -> U {
        self.dimensions.0
    }

    /// The height of the region.
    pub fn height(&self) -> U {
        self.dimensions.1
    }

    /// The coordinate of the top edge of the region.
    pub fn top_edge(&self) -> U {
        self.anchor().y
    }

    /// The coordinate of the bottom edge of the region.
    pub fn bottom_edge(&self) -> U {
        self.anchor().y + self.height()
    }

    /// The coordinate of the left edge of the region.
    pub fn left_edge(&self) -> U {
        self.anchor().x
    }

    /// The coordinate of the right edge of the region.
    pub fn right_edge(&self) -> U {
        self.anchor().x + self.width()
    }

    /// Whether or not an area intersects another area.
    pub fn intersects(self, other: impl Into<Self>) -> bool {
        let other = other.into();
        self.left_edge() < other.right_edge()
            && self.right_edge() > other.left_edge()
            && self.top_edge() < other.bottom_edge()
            && self.bottom_edge() > other.top_edge()
    }

    /// Whether or not an area wholly contains another area.
    pub fn contains(self, other: impl Into<Self>) -> bool {
        let other = other.into();
        other.right_edge() <= self.right_edge()
            && other.left_edge() >= self.left_edge()
            && other.top_edge() >= self.top_edge()
            && other.bottom_edge() <= self.bottom_edge()
    }

    /// Whether or not an area contains a point.
    pub fn contains_pt(self, pt: impl Into<Point<U>>) -> bool {
        self.contains(Self::unit().at(pt))
    }

    // NB: The center point is an integer and thus rounded, i.e. a 2x2 region at (0,0) has a center
    // at (0,0), when in reality the center would be at (0.5, 0.5).
    pub(crate) fn center_pt(&self) -> Point<U> {
        self.anchor()
            + Point {
                x: self.width() / Self::two(),
                y: self.height() / Self::two(),
            }
    }

    pub(crate) fn dimensions(&self) -> (U, U) {
        self.dimensions
    }

    // Strongly-typed alias for U::one() + U::One()
    fn two() -> U {
        U::one() + U::one()
    }
}

impl<P, U> From<(P, (U, U))> for Area<U>
where
    P: Into<Point<U>>,
    U: PrimInt + Default + PartialOrd,
{
    fn from((anchor, (width, height)): (P, (U, U))) -> Self {
        Self::new(width, height).at(anchor)
    }
}

impl<P, U> From<P> for Area<U>
where
    P: Into<Point<U>>,
    U: PrimInt + Default + PartialOrd,
{
    fn from(anchor: P) -> Self {
        Self::unit().at(anchor)
    }
}

/// A type representing a point in space. Should be passed by value.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct Point<U> {
    pub x: U, // The x-coordinate of the point.
    pub y: U, // The y-coordinate of the point.
}

impl<U> Debug for Point<U>
where
    U: PrimInt + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}x{:?}", self.x, self.y)
    }
}

impl<U> From<(U, U)> for Point<U>
where
    U: PrimInt,
{
    fn from((x, y): (U, U)) -> Self {
        Self { x, y }
    }
}

impl<U> From<&(U, U)> for Point<U>
where
    U: PrimInt,
{
    fn from((x, y): &(U, U)) -> Self {
        Self { x: *x, y: *y }
    }
}

impl<U> From<Point<U>> for (U, U)
where
    U: PrimInt,
{
    fn from(value: Point<U>) -> Self {
        (value.x, value.y)
    }
}

impl<U> Add for Point<U>
where
    U: PrimInt,
{
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x.saturating_add(other.x),
            y: self.y.saturating_add(other.y),
        }
    }
}

impl<U> Sub for Point<U>
where
    U: PrimInt,
{
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x.saturating_sub(other.x),
            y: self.y.saturating_sub(other.y),
        }
    }
}
