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

use {crate::area::Area, num::PrimInt};

// d888888b d8888b.  .d8b.  db    db d88888b d8888b. .d8888.  .d8b.  db
// `~~88~~' 88  `8D d8' `8b 88    88 88'     88  `8D 88'  YP d8' `8b 88
//    88    88oobY' 88ooo88 Y8    8P 88ooooo 88oobY' `8bo.   88ooo88 88
//    88    88`8b   88~~~88 `8b  d8' 88~~~~~ 88`8b     `Y8b. 88~~~88 88
//    88    88 `88. 88   88  `8bd8'  88.     88 `88. db   8D 88   88 88booo.
//    YP    88   YD YP   YP    YP    Y88888P 88   YD `8888Y' YP   YP Y88888P

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Traversal {
    Overlapping,
    Strict,
}

impl Traversal {
    pub(crate) fn eval<U>(&self, bounding_box: Area<U>, query_region: Area<U>) -> bool
    where
        U: PrimInt + std::default::Default,
    {
        match self {
            Traversal::Overlapping => query_region.intersects(bounding_box),
            Traversal::Strict => query_region.contains(bounding_box),
        }
    }
}
