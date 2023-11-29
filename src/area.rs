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

//! A rectangular region in the tree.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use {
    crate::point,
    num::PrimInt,
    std::{cmp::PartialOrd, default::Default, fmt::Debug},
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
#[derive(PartialEq, Eq, Clone, Copy, Hash, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct Area<U>
where
    U: PrimInt + Default + PartialOrd,
{
    #[builder(setter(into))]
    anchor: point::Point<U>,
    #[builder(default = "(U::one(), U::one())")]
    dimensions: (U, U),
}

impl<U> AreaBuilder<U>
where
    U: PrimInt + Default + PartialOrd,
{
    fn validate(&self) -> Result<(), String> {
        if let Some((w, h)) = self.dimensions {
            if w <= U::zero() {
                return Err("Areas may not have nonpositive widths.".to_string());
            }
            if h <= U::zero() {
                return Err("Areas may not have nonpositive heights.".to_string());
            }
        }
        Ok(())
    }
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
    /// The top-left coordinate (anchor) of the region.
    pub fn anchor(&self) -> point::Point<U> {
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
        self.anchor().y()
    }

    /// The coordinate of the bottom edge of the region.
    pub fn bottom_edge(&self) -> U {
        self.anchor().y() + self.height()
    }

    /// The coordinate of the left edge of the region.
    pub fn left_edge(&self) -> U {
        self.anchor().x()
    }

    /// The coordinate of the right edge of the region.
    pub fn right_edge(&self) -> U {
        self.anchor().x() + self.width()
    }

    /// Whether or not an area intersects another area.
    pub fn intersects(self, other: Self) -> bool {
        self.left_edge() < other.right_edge()
            && self.right_edge() > other.left_edge()
            && self.top_edge() < other.bottom_edge()
            && self.bottom_edge() > other.top_edge()
    }

    /// Whether or not an area wholly contains another area.
    pub fn contains(self, other: Self) -> bool {
        other.right_edge() <= self.right_edge()
            && other.left_edge() >= self.left_edge()
            && other.top_edge() >= self.top_edge()
            && other.bottom_edge() <= self.bottom_edge()
    }

    /// Whether or not an area contains a point.
    pub fn contains_pt(self, pt: point::Point<U>) -> bool {
        self.contains(
            AreaBuilder::default()
                .anchor(pt)
                .dimensions((U::one(), U::one()))
                .build()
                .expect("Unexpected error in Area::contains_pt."),
        )
    }

    // NB: The center point is an integer and thus rounded, i.e. a 2x2 region at (0,0) has a center
    // at (0,0), when in reality the center would be at (0.5, 0.5).
    pub(crate) fn center_pt(&self) -> point::Point<U> {
        self.anchor()
            + point::Point {
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
