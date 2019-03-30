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

// For testing .query(), .modify().
mod query_tests {
    use {
        crate::util::unordered_elements_are,
        quadtree_rs::{area::AreaBuilder, Quadtree},
    };

    #[test]
    fn query_empty() {
        let qt = Quadtree::<u32, u8>::new(2);
        let mut iter = qt.query(
            AreaBuilder::default()
                .anchor((0, 0).into())
                .dimensions((4, 4))
                .build()
                .unwrap(),
        );
        debug_assert_eq!(iter.next(), None);
    }

    #[test]
    fn query_on_point() {
        let mut qt = Quadtree::<u32, u8>::new(1);
        assert!(qt
            .insert(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .build()
                    .unwrap(),
                49,
            )
            .is_some());

        // Requesting a region which does contain '49'.
        let mut iter1 = qt.query(
            AreaBuilder::default()
                .anchor((0, 0).into())
                .build()
                .unwrap(),
        );
        let entry = iter1.next().unwrap();
        let entry_area = entry.area();

        debug_assert_eq!(entry_area.anchor().x(), 0);
        debug_assert_eq!(entry_area.anchor().y(), 0);
        debug_assert_eq!(entry_area.width(), 1);
        debug_assert_eq!(entry_area.height(), 1);

        debug_assert_eq!(entry.value_ref(), &49);
        debug_assert_eq!(iter1.next(), None);

        // Requesting regions which don't contain '49'.
        let mut iter2 = qt.query(
            AreaBuilder::default()
                .anchor((0, 1).into())
                .build()
                .unwrap(),
        );
        debug_assert_eq!(iter2.next(), None);

        let mut iter3 = qt.query(
            AreaBuilder::default()
                .anchor((1, 0).into())
                .build()
                .unwrap(),
        );
        debug_assert_eq!(iter3.next(), None);

        let mut iter4 = qt.query(
            AreaBuilder::default()
                .anchor((1, 1).into())
                .build()
                .unwrap(),
        );
        debug_assert_eq!(iter4.next(), None);
    }

    #[test]
    fn query_in_region() {
        let mut qt = Quadtree::<u32, u8>::new(4);
        //   0  1  2  3  4  5  6
        // 0 +--+--+--+--+--+--+
        //   |  |  |  |  |  |  |
        // 1 +--+--+--+--+--+--+
        //   |  |  |  |  |  |  |
        // 2 +--+--o o o o--+--+  o @ (2, 2)->(2x2) #10
        //   |  |   o o o   |  |  x @ (3, 3)->(2x2) #55
        // 3 +--+--o oxoxox x--+
        //   |  |   o oxox x   |
        // 4 +--+--o oxoxox x--+
        //   |  |  |   x x x   |
        // 5 +--+--+--x x x x--+
        //   |  |  |  |  |  |  |
        // 6 +--+--+--+--+--+--+
        assert!(qt
            .insert(
                AreaBuilder::default()
                    .anchor((2, 2).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap(),
                10,
            )
            .is_some());
        assert!(qt
            .insert(
                AreaBuilder::default()
                    .anchor((3, 3).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap(),
                55,
            )
            .is_some());

        // Queries which turn up empty:
        let mut empty1 = qt.query(
            AreaBuilder::default()
                .anchor((1, 1).into())
                .build()
                .unwrap(),
        );
        debug_assert_eq!(empty1.next(), None);

        let mut empty2 = qt.query(
            AreaBuilder::default()
                .anchor((0, 0).into())
                .dimensions((2, 2))
                .build()
                .unwrap(),
        );
        debug_assert_eq!(empty2.next(), None);

        let mut empty3 = qt.query(
            AreaBuilder::default()
                .anchor((0, 0).into())
                .dimensions((6, 2))
                .build()
                .unwrap(),
        );
        debug_assert_eq!(empty3.next(), None);

        let mut empty4 = qt.query(
            AreaBuilder::default()
                .anchor((0, 0).into())
                .dimensions((2, 6))
                .build()
                .unwrap(),
        );
        debug_assert_eq!(empty4.next(), None);

        // Queries which capture #10:
        let mut ten1 = qt.query(
            AreaBuilder::default()
                .anchor((2, 2).into())
                .build()
                .unwrap(),
        );
        debug_assert_eq!(ten1.next().unwrap().value_ref(), &10);
        debug_assert_eq!(ten1.next(), None);

        let mut ten2 = qt.query(
            AreaBuilder::default()
                .anchor((2, 3).into())
                .build()
                .unwrap(),
        );
        debug_assert_eq!(ten2.next().unwrap().value_ref(), &10);
        debug_assert_eq!(ten2.next(), None);

        let mut ten3 = qt.query(
            AreaBuilder::default()
                .anchor((3, 2).into())
                .build()
                .unwrap(),
        );
        debug_assert_eq!(ten3.next().unwrap().value_ref(), &10);
        debug_assert_eq!(ten3.next(), None);

        // Queries which capture #10 but are larger than 1x1.
        let mut ten4 = qt.query(
            AreaBuilder::default()
                .anchor((2, 2).into())
                .dimensions((2, 1))
                .build()
                .unwrap(),
        );
        debug_assert_eq!(ten4.next().unwrap().value_ref(), &10);
        debug_assert_eq!(ten4.next(), None);

        let mut ten5 = qt.query(
            AreaBuilder::default()
                .anchor((2, 2).into())
                .build()
                .unwrap(),
        );
        debug_assert_eq!(ten5.next().unwrap().value_ref(), &10);
        debug_assert_eq!(ten5.next(), None);

        // Queries which capture #55:
        let mut fiftyfive1 = qt.query(
            AreaBuilder::default()
                .anchor((3, 4).into())
                .build()
                .unwrap(),
        );
        debug_assert_eq!(fiftyfive1.next().unwrap().value_ref(), &55);
        debug_assert_eq!(fiftyfive1.next(), None);

        let mut fiftyfive2 = qt.query(
            AreaBuilder::default()
                .anchor((4, 3).into())
                .build()
                .unwrap(),
        );
        debug_assert_eq!(fiftyfive2.next().unwrap().value_ref(), &55);
        debug_assert_eq!(fiftyfive2.next(), None);

        let mut fiftyfive3 = qt.query(
            AreaBuilder::default()
                .anchor((4, 4).into())
                .build()
                .unwrap(),
        );
        debug_assert_eq!(fiftyfive3.next().unwrap().value_ref(), &55);
        debug_assert_eq!(fiftyfive3.next(), None);

        // Queries which capture #55 but are larger than 1x1.

        let mut fiftyfive4 = qt.query(
            AreaBuilder::default()
                .anchor((4, 3).into())
                .build()
                .unwrap(),
        );
        debug_assert_eq!(fiftyfive4.next().unwrap().value_ref(), &55);
        debug_assert_eq!(fiftyfive4.next(), None);

        let mut fiftyfive5 = qt.query(
            AreaBuilder::default()
                .anchor((3, 4).into())
                .dimensions((2, 2))
                .build()
                .unwrap(),
        );
        debug_assert_eq!(fiftyfive5.next().unwrap().value_ref(), &55);
        debug_assert_eq!(fiftyfive5.next(), None);

        // Queries which capture both #10 and #55. Dunno in what order.

        debug_assert!(unordered_elements_are(
            qt.query(
                AreaBuilder::default()
                    .anchor((3, 3).into())
                    .build()
                    .unwrap()
            )
            .map(|e| e.value_ref()),
            vec![&10, &55],
        ));

        debug_assert!(unordered_elements_are(
            qt.query(
                AreaBuilder::default()
                    .anchor((3, 3).into())
                    .dimensions((3, 3))
                    .build()
                    .unwrap()
            )
            .map(|e| e.value_ref()),
            vec![&10, &55],
        ));

        debug_assert!(unordered_elements_are(
            qt.query(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .dimensions((6, 6))
                    .build()
                    .unwrap()
            )
            .map(|e| e.value_ref()),
            vec![&10, &55],
        ));

        debug_assert!(unordered_elements_are(
            qt.query(
                AreaBuilder::default()
                    .anchor((2, 2).into())
                    .dimensions((6, 6))
                    .build()
                    .unwrap()
            )
            .map(|e| e.value_ref()),
            vec![&10, &55],
        ));

        debug_assert!(unordered_elements_are(
            qt.query(
                AreaBuilder::default()
                    .anchor((2, 2).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            )
            .map(|e| e.value_ref()),
            vec![&10, &55],
        ));
    }

    #[test]
    fn query_strict_in_region() {
        let mut qt = Quadtree::<u32, u8>::new(4);
        //   0  1  2  3  4  5  6
        // 0 +--+--+--+--+--+--+
        //   |  |  |  |  |  |  |
        // 1 +--+--+--+--+--+--+
        //   |  |  |  |  |  |  |
        // 2 +--+--o o o o--+--+  o @ (2, 2)->(2x2) #10
        //   |  |   o o o   |  |  x @ (3, 3)->(2x2) #55
        // 3 +--+--o oxoxox x--+
        //   |  |   o oxox x   |
        // 4 +--+--o oxoxox x--+
        //   |  |  |   x x x   |
        // 5 +--+--+--x x x x--+
        //   |  |  |  |  |  |  |
        // 6 +--+--+--+--+--+--+
        assert!(qt
            .insert(
                AreaBuilder::default()
                    .anchor((2, 2).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap(),
                10,
            )
            .is_some());
        assert!(qt
            .insert(
                AreaBuilder::default()
                    .anchor((3, 3).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap(),
                55,
            )
            .is_some());

        // Queries which turn up empty:
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((1, 1).into())
                    .build()
                    .unwrap()
            )
            .next(),
            None
        );
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            )
            .next(),
            None
        );
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .dimensions((6, 2))
                    .build()
                    .unwrap()
            )
            .next(),
            None
        );
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .dimensions((2, 6))
                    .build()
                    .unwrap()
            )
            .next(),
            None
        );

        // Queries which capture portions of #10 but not enough.
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((2, 2).into())
                    .build()
                    .unwrap()
            )
            .next(),
            None
        );
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((2, 3).into())
                    .build()
                    .unwrap()
            )
            .next(),
            None
        );
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((3, 2).into())
                    .build()
                    .unwrap()
            )
            .next(),
            None
        );
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((2, 2).into())
                    .dimensions((2, 1))
                    .build()
                    .unwrap()
            )
            .next(),
            None
        );
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((2, 2).into())
                    .build()
                    .unwrap()
            )
            .next(),
            None
        );

        // Queries which capture portions of #55 but not enough.
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((3, 4).into())
                    .build()
                    .unwrap()
            )
            .next(),
            None
        );
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((4, 3).into())
                    .build()
                    .unwrap()
            )
            .next(),
            None
        );
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((4, 4).into())
                    .build()
                    .unwrap()
            )
            .next(),
            None
        );
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((4, 3).into())
                    .build()
                    .unwrap()
            )
            .next(),
            None
        );
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((3, 4).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            )
            .next(),
            None
        );

        // Queries which capture portions of both #10 and #55. but still aren't enough

        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((3, 3).into())
                    .build()
                    .unwrap()
            )
            .next(),
            None
        );

        // Queries which contain one of the other:
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((3, 3).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            )
            .next()
            .unwrap()
            .value_ref(),
            &55
        );
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((3, 3).into())
                    .dimensions((3, 3))
                    .build()
                    .unwrap()
            )
            .next()
            .unwrap()
            .value_ref(),
            &55
        );
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((3, 3).into())
                    .dimensions((4, 4))
                    .build()
                    .unwrap()
            )
            .next()
            .unwrap()
            .value_ref(),
            &55
        );
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .dimensions((4, 4))
                    .build()
                    .unwrap()
            )
            .next()
            .unwrap()
            .value_ref(),
            &10
        );
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((1, 1).into())
                    .dimensions((3, 3))
                    .build()
                    .unwrap()
            )
            .next()
            .unwrap()
            .value_ref(),
            &10
        );
        debug_assert_eq!(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((2, 2).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap()
            )
            .next()
            .unwrap()
            .value_ref(),
            &10
        );

        // A query which contains both:
        debug_assert!(unordered_elements_are(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .dimensions((6, 6))
                    .build()
                    .unwrap()
            )
            .map(|e| e.value_ref()),
            vec![&10, &55]
        ));
        debug_assert!(unordered_elements_are(
            qt.query_strict(
                AreaBuilder::default()
                    .anchor((2, 2).into())
                    .dimensions((6, 6))
                    .build()
                    .unwrap()
            )
            .map(|e| e.value_ref()),
            vec![&10, &55]
        ));
    }

    #[test]
    fn query_exhibiting_collection() {
        let mut qt: Quadtree<u8, f32> = Quadtree::new(2);
        assert!(qt
            .insert(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .dimensions((2, 2))
                    .build()
                    .unwrap(),
                1.234,
            )
            .is_some());

        let mut query_obj = qt.query(
            AreaBuilder::default()
                .anchor((0, 0).into())
                .build()
                .unwrap(),
        );

        debug_assert_eq!(query_obj.next().unwrap().value_ref(), &1.234);
    }

    #[test]
    fn modify_empty() {
        // Modification shouldn't change the emptiness.
        let mut qt = Quadtree::<u32, u8>::new(2);
        qt.modify(
            AreaBuilder::default()
                .anchor((0, 0).into())
                .dimensions((4, 4))
                .build()
                .unwrap(),
            |v| *v *= 2,
        );
        debug_assert!(qt.is_empty());
    }

    #[test]
    fn modify() {
        let mut qt = Quadtree::<u32, u8>::new(3);

        // Insert #49 at (0, 0)->1x1.
        assert!(qt
            .insert(
                AreaBuilder::default()
                    .anchor((0, 0).into())
                    .build()
                    .unwrap(),
                49,
            )
            .is_some());
        qt.modify(
            AreaBuilder::default()
                .anchor((0, 0).into())
                .build()
                .unwrap(),
            |i| *i += 1,
        );

        // And verify.
        let mut tmp_iter_1 = qt.query(
            AreaBuilder::default()
                .anchor((0, 0).into())
                .build()
                .unwrap(),
        );
        debug_assert_eq!(tmp_iter_1.next().unwrap().value_ref(), &50);
        debug_assert_eq!(tmp_iter_1.next(), None);

        // Insert #17 at (2, 2)->3x3.
        assert!(qt
            .insert(
                AreaBuilder::default()
                    .anchor((2, 2).into())
                    .dimensions((3, 3))
                    .build()
                    .unwrap(),
                17,
            )
            .is_some());

        // Up it to 18,
        qt.modify(
            AreaBuilder::default()
                .anchor((1, 1).into())
                .dimensions((2, 2))
                .build()
                .unwrap(),
            |i| *i += 1,
        );
        // And verify.
        let mut tmp_iter_2 = qt.query(
            AreaBuilder::default()
                .anchor((2, 2).into())
                .build()
                .unwrap(),
        );
        debug_assert_eq!(tmp_iter_2.next().unwrap().value_ref(), &18);
        debug_assert_eq!(tmp_iter_2.next(), None);

        // Reset everything in (0, 0)->6x6 to "0".
        qt.modify(
            AreaBuilder::default()
                .anchor((0, 0).into())
                .dimensions((6, 6))
                .build()
                .unwrap(),
            |i| *i = 0,
        );
        // Every value is now 0.

        for entry in qt.query(
            AreaBuilder::default()
                .anchor((0, 0).into())
                .dimensions((6, 6))
                .build()
                .unwrap(),
        ) {
            debug_assert_eq!(entry.value_ref(), &0);
        }
    }
}
