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

mod util; // For unordered_elements_are.

// For testing .iter(), .iter_mut(), .regions(), .values(), .values_mut().
mod iterator_tests {
    use {
        crate::util::unordered_elements_are,
        quadtree_rs::{area::AreaBuilder, entry::Entry, Quadtree},
    };

    fn mk_quadtree_for_iter_tests() -> Quadtree<i32, i8> {
        let mut qt = Quadtree::<i32, i8>::new_with_anchor((-35, -35).into(), 8);
        qt.extend(vec![((0, -5), 10), ((-15, 20), -25), ((30, -35), 40)]);
        qt
    }

    #[test]
    fn iter_all() {
        let qt = mk_quadtree_for_iter_tests();

        debug_assert!(unordered_elements_are(
            qt.iter().map(|e| e.value_ref()),
            vec![&-25, &10, &40]
        ));
    }

    // The same as iter_all(), except we mutate each value by +1.
    #[test]
    fn iter_mut_all() {
        let mut qt = mk_quadtree_for_iter_tests();

        qt.modify_all(|v| *v += 1);

        debug_assert!(unordered_elements_are(
            qt.iter().map(|e| e.value_ref()),
            vec![&-24, &11, &41]
        ));
    }

    #[test]
    fn regions() {
        let qt = mk_quadtree_for_iter_tests();
        debug_assert!(unordered_elements_are(
            qt.regions().map(|a| a.into()),
            vec![((0, -5), (1, 1)), ((-15, 20), (1, 1)), ((30, -35), (1, 1))],
        ));
    }

    #[test]
    fn values() {
        let qt = mk_quadtree_for_iter_tests();

        debug_assert!(unordered_elements_are(qt.values(), vec![&10, &-25, &40]));
    }

    #[test]
    fn into_iterator_reference() {
        let mut qt = mk_quadtree_for_iter_tests();
        let entries: Vec<&Entry<i32, i8>> = (&qt).into_iter().collect();
        debug_assert!(unordered_elements_are(
            entries.iter().map(|e| e.value_ref()),
            vec![&10, &-25, &40],
        ));

        qt.reset();
        debug_assert!(qt.is_empty());
    }

    #[test]
    fn delete_everything() {
        let mut qt = mk_quadtree_for_iter_tests();
        debug_assert_eq!(qt.len(), 3);
        qt.delete(
            AreaBuilder::default()
                .anchor((-35, -35).into())
                .dimensions((80, 80))
                .build()
                .unwrap(),
        );
        debug_assert_eq!(qt.len(), 0);
    }

    #[test]
    fn delete_region() {
        let mut qt = mk_quadtree_for_iter_tests();
        debug_assert_eq!(qt.len(), 3);
        // Near miss.
        qt.delete(
            AreaBuilder::default()
                .anchor((29, -36).into())
                .build()
                .unwrap(),
        );
        debug_assert_eq!(qt.len(), 3);

        // Direct hit!
        let mut returned_entries = qt.delete(
            AreaBuilder::default()
                .anchor((30, -35).into())
                .build()
                .unwrap(),
        );
        debug_assert_eq!(qt.len(), 2);
        let hit = returned_entries.next().unwrap();
        debug_assert_eq!(hit.value_ref(), &40);
        debug_assert_eq!(
            hit.area(),
            AreaBuilder::default()
                .anchor((30, -35).into())
                .build()
                .unwrap()
        );
    }

    #[test]
    fn delete_region_two() {
        let mut qt = mk_quadtree_for_iter_tests();
        debug_assert_eq!(qt.len(), 3);

        // Just large enough to encompass the two points.
        let returned_entries: Vec<Entry<i32, i8>> = qt
            .delete(
                AreaBuilder::default()
                    .anchor((-15, -5).into())
                    .dimensions((16, 26))
                    .build()
                    .unwrap(),
            )
            .collect();
        debug_assert_eq!(qt.len(), 1);

        debug_assert!(unordered_elements_are(
            returned_entries.iter().map(|e| e.value_ref()),
            vec![&-25, &10,]
        ));
    }
}
