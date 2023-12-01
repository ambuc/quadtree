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

use crate::geometry::Area;
use num::PrimInt;
use std::default::Default;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Traversal {
    Overlapping,
    Strict,
}

impl Traversal {
    pub(crate) fn eval<U>(self, bounding_box: Area<U>, query_region: Area<U>) -> bool
    where
        U: PrimInt + Default,
    {
        match self {
            Traversal::Overlapping => query_region.intersects(bounding_box),
            Traversal::Strict => query_region.contains(bounding_box),
        }
    }
}
