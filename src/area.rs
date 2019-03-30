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

//! A type representing an area in space.

use {
    crate::point,
    num::PrimInt,
    std::{cmp::PartialOrd, default::Default, fmt::Debug},
};

//  .d8b.  d8888b. d88888b  .d8b.
//  d8' `8b 88  `8D 88'     d8' `8b
//  88ooo88 88oobY' 88ooooo 88ooo88
//  88~~~88 88`8b   88~~~~~ 88~~~88
//  88   88 88 `88. 88.     88   88
//  YP   YP 88   YD Y88888P YP   YP

// Transparent alias. In docs and user-facing APIs, this resolves to ((U, U), (U, U)).
pub(crate) type Type<U> = (point::Type<U>, (U, U));

/// Lightweight data type to represent a region.
///   - The top-left anchor may be positive or negative in either coordinate.
///   - Defined by a top-left anchor and a width/height.
///   - The width/height must both be positive and nonzero.
///   - Should be passed by value.
#[derive(PartialEq, Eq, Clone, Copy, Hash, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct Area<U>
where
    U: PrimInt + Default + PartialOrd,
{
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

impl<U> Into<Type<U>> for Area<U>
where
    U: PrimInt + Default,
{
    fn into(self) -> Type<U> {
        (self.anchor.into(), self.dimensions())
    }
}

impl<U> Area<U>
where
    U: PrimInt + Default,
{
    // pub

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

    // pub(crate)

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

    // fn

    // Strongly-typed alias for U::one() + U::One()
    fn two() -> U {
        U::one() + U::one()
    }
}
