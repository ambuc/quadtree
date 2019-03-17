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
    use {crate::util::unordered_elements_are, quadtree_rs::Quadtree};

    #[test]
    fn query_empty() {
        let qt = Quadtree::<u32, u8>::new(2);
        let mut iter = qt.query((0, 0), (4, 4));
        debug_assert_eq!(iter.next(), None);
    }

    #[test]
    fn query_on_point() {
        let mut qt = Quadtree::<u32, u8>::new(1);
        qt.insert((0, 0), (1, 1), 49);

        // Requesting a region which does contain '49'.
        let mut iter1 = qt.query((0, 0), (1, 1));
        let entry = iter1.next().unwrap();
        debug_assert_eq!(entry.region(), ((0, 0), (1, 1)));
        debug_assert_eq!(entry.value_ref(), &49);
        debug_assert_eq!(iter1.next(), None);

        // Requesting regions which don't contain '49'.
        let mut iter2 = qt.query((0, 1), (1, 1));
        debug_assert_eq!(iter2.next(), None);

        let mut iter3 = qt.query((1, 0), (1, 1));
        debug_assert_eq!(iter3.next(), None);

        let mut iter4 = qt.query((1, 1), (1, 1));
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
        qt.insert((2, 2), (2, 2), 10);
        qt.insert((3, 3), (2, 2), 55);

        // Queries which turn up empty:
        let mut empty1 = qt.query((1, 1), (1, 1));
        debug_assert_eq!(empty1.next(), None);

        let mut empty2 = qt.query((0, 0), (2, 2));
        debug_assert_eq!(empty2.next(), None);

        let mut empty3 = qt.query((0, 0), (6, 2));
        debug_assert_eq!(empty3.next(), None);

        let mut empty4 = qt.query((0, 0), (2, 6));
        debug_assert_eq!(empty4.next(), None);

        // Queries which capture #10:
        let mut ten1 = qt.query((2, 2), (1, 1));
        debug_assert_eq!(ten1.next().unwrap().value_ref(), &10);
        debug_assert_eq!(ten1.next(), None);

        let mut ten2 = qt.query((2, 3), (1, 1));
        debug_assert_eq!(ten2.next().unwrap().value_ref(), &10);
        debug_assert_eq!(ten2.next(), None);

        let mut ten3 = qt.query((3, 2), (1, 1));
        debug_assert_eq!(ten3.next().unwrap().value_ref(), &10);
        debug_assert_eq!(ten3.next(), None);

        // Queries which capture #10 but are larger than 1x1.
        let mut ten4 = qt.query((2, 2), (2, 1));
        debug_assert_eq!(ten4.next().unwrap().value_ref(), &10);
        debug_assert_eq!(ten4.next(), None);

        let mut ten5 = qt.query((2, 2), (1, 2));
        debug_assert_eq!(ten5.next().unwrap().value_ref(), &10);
        debug_assert_eq!(ten5.next(), None);

        // Queries which capture #55:
        let mut fiftyfive1 = qt.query((3, 4), (1, 1));
        debug_assert_eq!(fiftyfive1.next().unwrap().value_ref(), &55);
        debug_assert_eq!(fiftyfive1.next(), None);

        let mut fiftyfive2 = qt.query((4, 3), (1, 1));
        debug_assert_eq!(fiftyfive2.next().unwrap().value_ref(), &55);
        debug_assert_eq!(fiftyfive2.next(), None);

        let mut fiftyfive3 = qt.query((4, 4), (1, 1));
        debug_assert_eq!(fiftyfive3.next().unwrap().value_ref(), &55);
        debug_assert_eq!(fiftyfive3.next(), None);

        // Queries which capture #55 but are larger than 1x1.

        let mut fiftyfive4 = qt.query((4, 3), (1, 2));
        debug_assert_eq!(fiftyfive4.next().unwrap().value_ref(), &55);
        debug_assert_eq!(fiftyfive4.next(), None);

        let mut fiftyfive5 = qt.query((3, 4), (2, 2));
        debug_assert_eq!(fiftyfive5.next().unwrap().value_ref(), &55);
        debug_assert_eq!(fiftyfive5.next(), None);

        // Queries which capture both #10 and #55. Dunno in what order.

        debug_assert!(unordered_elements_are(
            qt.query((3, 3), (1, 1)).map(|e| e.value_ref()),
            vec![&10, &55],
        ));

        debug_assert!(unordered_elements_are(
            qt.query((3, 3), (3, 3)).map(|e| e.value_ref()),
            vec![&10, &55],
        ));

        debug_assert!(unordered_elements_are(
            qt.query((0, 0), (6, 6)).map(|e| e.value_ref()),
            vec![&10, &55],
        ));

        debug_assert!(unordered_elements_are(
            qt.query((2, 2), (6, 6)).map(|e| e.value_ref()),
            vec![&10, &55],
        ));

        debug_assert!(unordered_elements_are(
            qt.query((2, 2), (2, 2)).map(|e| e.value_ref()),
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
        qt.insert((2, 2), (2, 2), 10);
        qt.insert((3, 3), (2, 2), 55);

        // Queries which turn up empty:
        debug_assert_eq!(qt.query_strict((1, 1), (1, 1)).next(), None);
        debug_assert_eq!(qt.query_strict((0, 0), (2, 2)).next(), None);
        debug_assert_eq!(qt.query_strict((0, 0), (6, 2)).next(), None);
        debug_assert_eq!(qt.query_strict((0, 0), (2, 6)).next(), None);

        // Queries which capture portions of #10 but not enough.
        debug_assert_eq!(qt.query_strict((2, 2), (1, 1)).next(), None);
        debug_assert_eq!(qt.query_strict((2, 3), (1, 1)).next(), None);
        debug_assert_eq!(qt.query_strict((3, 2), (1, 1)).next(), None);
        debug_assert_eq!(qt.query_strict((2, 2), (2, 1)).next(), None);
        debug_assert_eq!(qt.query_strict((2, 2), (1, 2)).next(), None);

        // Queries which capture portions of #55 but not enough.
        debug_assert_eq!(qt.query_strict((3, 4), (1, 1)).next(), None);
        debug_assert_eq!(qt.query_strict((4, 3), (1, 1)).next(), None);
        debug_assert_eq!(qt.query_strict((4, 4), (1, 1)).next(), None);
        debug_assert_eq!(qt.query_strict((4, 3), (1, 2)).next(), None);
        debug_assert_eq!(qt.query_strict((3, 4), (2, 2)).next(), None);

        // Queries which capture portions of both #10 and #55. but still aren't enough

        debug_assert_eq!(qt.query_strict((3, 3), (1, 1)).next(), None);

        // Queries which contain one of the other:
        debug_assert_eq!(
            qt.query_strict((3, 3), (2, 2)).next().unwrap().value_ref(),
            &55
        );
        debug_assert_eq!(
            qt.query_strict((3, 3), (3, 3)).next().unwrap().value_ref(),
            &55
        );
        debug_assert_eq!(
            qt.query_strict((3, 3), (4, 4)).next().unwrap().value_ref(),
            &55
        );
        debug_assert_eq!(
            qt.query_strict((0, 0), (4, 4)).next().unwrap().value_ref(),
            &10
        );
        debug_assert_eq!(
            qt.query_strict((1, 1), (3, 3)).next().unwrap().value_ref(),
            &10
        );
        debug_assert_eq!(
            qt.query_strict((2, 2), (2, 2)).next().unwrap().value_ref(),
            &10
        );

        // A query which contains both:
        debug_assert!(unordered_elements_are(
            qt.query_strict((0, 0), (6, 6)).map(|e| e.value_ref()),
            vec![&10, &55]
        ));
        debug_assert!(unordered_elements_are(
            qt.query_strict((2, 2), (6, 6)).map(|e| e.value_ref()),
            vec![&10, &55]
        ));
    }

    #[test]
    fn query_exhibiting_collection() {
        let mut qt: Quadtree<u8, f32> = Quadtree::new(2);
        qt.insert((0, 0), (2, 2), 1.234);

        let mut query_obj = qt.query((0, 0), (1, 1));

        debug_assert_eq!(query_obj.next().unwrap().value_ref(), &1.234);
    }

    #[test]
    fn modify_empty() {
        // Modification shouldn't change the emptiness.
        let mut qt = Quadtree::<u32, u8>::new(2);
        qt.modify((0, 0), (4, 4), |v| *v *= 2);
        debug_assert!(qt.is_empty());
    }

    #[test]
    fn modify() {
        let mut qt = Quadtree::<u32, u8>::new(3);

        // Insert #49 at (0, 0)->1x1.
        qt.insert((0, 0), (1, 1), 49);
        qt.modify((0, 0), (1, 1), |i| *i += 1);

        // And verify.
        let mut tmp_iter_1 = qt.query((0, 0), (1, 1));
        debug_assert_eq!(tmp_iter_1.next().unwrap().value_ref(), &50);
        debug_assert_eq!(tmp_iter_1.next(), None);

        // Insert #17 at (2, 2)->3x3.
        qt.insert((2, 2), (3, 3), 17);
        // Up it to 18,
        qt.modify((1, 1), (2, 2), |i| *i += 1);
        // And verify.
        let mut tmp_iter_2 = qt.query((2, 2), (1, 1));
        debug_assert_eq!(tmp_iter_2.next().unwrap().value_ref(), &18);
        debug_assert_eq!(tmp_iter_2.next(), None);

        // Reset everything in (0, 0)->6x6 to "0".
        qt.modify((0, 0), (6, 6), |i| *i = 0);
        // Every value is now 0.

        for entry in qt.query((0, 0), (6, 6)) {
            debug_assert_eq!(entry.value_ref(), &0);
        }
    }
}
