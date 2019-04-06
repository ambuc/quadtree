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

// d888888b db    db d8888b. d88888b .d8888.
// `~~88~~' `8b  d8' 88  `8D 88'     88'  YP
//    88     `8bd8'  88oodD' 88ooooo `8bo.
//    88       88    88~~~   88~~~~~   `Y8b.
//    88       88    88      88.     db   8D
//    YP       YP    88      Y88888P `8888Y'

// The hashmap storage type for qtinners. Made explicit here for brevity in other files.
pub(crate) type StoreType<U, V> = std::collections::HashMap<u64, crate::entry::Entry<U, V>>;
