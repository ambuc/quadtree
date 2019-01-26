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

// For testing .query(), .query_mut() over different regions.
mod query_tests {
    use crate::util::unordered_elements_are;
    use quadtree_impl::Quadtree;

    #[test]
    fn query_empty() {
        let q = Quadtree::<u32, u8>::new(2);
        let mut iter = q.query((0, 0), (4, 4));
        debug_assert_eq!(iter.size_hint(), (0, Some(0)));
        debug_assert_eq!(iter.next(), None);
        debug_assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    #[test]
    fn query_on_point() {
        let mut q = Quadtree::<u32, u8>::new(1);
        q.insert((0, 0), (1, 1), 49);

        // Requesting a region which does contain '49'.
        let mut iter1 = q.query((0, 0), (1, 1));
        debug_assert_eq!(iter1.size_hint(), (0, Some(1)));
        debug_assert_eq!(iter1.next(), Some((&((0, 0), (1, 1)), &49)));
        debug_assert_eq!(iter1.size_hint(), (0, Some(0)));
        debug_assert_eq!(iter1.next(), None);
        debug_assert_eq!(iter1.size_hint(), (0, Some(0)));

        // Requesting regions which don't contain '49'.
        let mut iter2 = q.query((0, 1), (1, 1));
        debug_assert_eq!(iter2.size_hint(), (0, Some(1)));
        debug_assert_eq!(iter2.next(), None);
        debug_assert_eq!(iter2.size_hint(), (0, Some(0)));

        let mut iter3 = q.query((1, 0), (1, 1));
        debug_assert_eq!(iter3.size_hint(), (0, Some(1)));
        debug_assert_eq!(iter3.next(), None);
        debug_assert_eq!(iter3.size_hint(), (0, Some(0)));

        let mut iter4 = q.query((1, 1), (1, 1));
        debug_assert_eq!(iter4.size_hint(), (0, Some(1)));
        debug_assert_eq!(iter4.next(), None);
        debug_assert_eq!(iter4.size_hint(), (0, Some(0)));
    }

    #[test]
    fn query_in_region() {
        let mut q = Quadtree::<u32, u8>::new(4);
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
        debug_assert!(q.insert((2, 2), (2, 2), 10));
        debug_assert!(q.insert((3, 3), (2, 2), 55));

        let expected_ten = Some((&((2, 2), (2, 2)), &10));
        let expected_fifty_five = Some((&((3, 3), (2, 2)), &55));

        // Queries which turn up empty:
        let mut empty1 = q.query((1, 1), (1, 1));
        debug_assert_eq!(empty1.size_hint(), (0, Some(2)));
        debug_assert_eq!(empty1.next(), None);
        debug_assert_eq!(empty1.size_hint(), (0, Some(0)));

        let mut empty2 = q.query((0, 0), (2, 2));
        debug_assert_eq!(empty2.size_hint(), (0, Some(2)));
        debug_assert_eq!(empty2.next(), None);
        debug_assert_eq!(empty2.size_hint(), (0, Some(0)));

        let mut empty3 = q.query((0, 0), (6, 2));
        debug_assert_eq!(empty3.size_hint(), (0, Some(2)));
        debug_assert_eq!(empty3.next(), None);
        debug_assert_eq!(empty3.size_hint(), (0, Some(0)));

        let mut empty4 = q.query((0, 0), (2, 6));
        debug_assert_eq!(empty4.size_hint(), (0, Some(2)));
        debug_assert_eq!(empty4.next(), None);
        debug_assert_eq!(empty4.size_hint(), (0, Some(0)));

        // Queries which capture #10:
        let mut ten1 = q.query((2, 2), (1, 1));
        debug_assert_eq!(ten1.next(), expected_ten);
        debug_assert_eq!(ten1.next(), None);

        let mut ten2 = q.query((2, 3), (1, 1));
        debug_assert_eq!(ten2.next(), expected_ten);
        debug_assert_eq!(ten2.next(), None);

        let mut ten3 = q.query((3, 2), (1, 1));
        debug_assert_eq!(ten3.next(), expected_ten);
        debug_assert_eq!(ten3.next(), None);

        // Queries which capture #10 but are larger than 1x1.
        let mut ten4 = q.query((2, 2), (2, 1));
        debug_assert_eq!(ten4.next(), expected_ten);
        debug_assert_eq!(ten4.next(), None);

        let mut ten5 = q.query((2, 2), (1, 2));
        debug_assert_eq!(ten5.next(), expected_ten);
        debug_assert_eq!(ten5.next(), None);

        // Queries which capture #55:
        let mut fiftyfive1 = q.query((3, 4), (1, 1));
        debug_assert_eq!(fiftyfive1.next(), expected_fifty_five);
        debug_assert_eq!(fiftyfive1.next(), None);

        let mut fiftyfive2 = q.query((4, 3), (1, 1));
        debug_assert_eq!(fiftyfive2.next(), expected_fifty_five);
        debug_assert_eq!(fiftyfive2.next(), None);

        let mut fiftyfive3 = q.query((4, 4), (1, 1));
        debug_assert_eq!(fiftyfive3.next(), expected_fifty_five);
        debug_assert_eq!(fiftyfive3.next(), None);

        // Queries which capture #55 but are larger than 1x1.

        let mut fiftyfive4 = q.query((4, 3), (1, 2));
        debug_assert_eq!(fiftyfive4.next(), expected_fifty_five);
        debug_assert_eq!(fiftyfive4.next(), None);

        let mut fiftyfive5 = q.query((3, 4), (2, 2));
        debug_assert_eq!(fiftyfive5.next(), expected_fifty_five);
        debug_assert_eq!(fiftyfive5.next(), None);

        // Queries which capture both #10 and #55. Dunno in what order.

        debug_assert!(unordered_elements_are(
            q.query((3, 3), (1, 1)),
            vec![(&((2, 2), (2, 2)), &10), (&((3, 3), (2, 2)), &55)]
        ));

        debug_assert!(unordered_elements_are(
            q.query((3, 3), (3, 3)),
            vec![(&((2, 2), (2, 2)), &10), (&((3, 3), (2, 2)), &55)]
        ));

        debug_assert!(unordered_elements_are(
            q.query((0, 0), (6, 6)),
            vec![(&((2, 2), (2, 2)), &10), (&((3, 3), (2, 2)), &55)]
        ));

        debug_assert!(unordered_elements_are(
            q.query((2, 2), (6, 6)),
            vec![(&((2, 2), (2, 2)), &10), (&((3, 3), (2, 2)), &55)]
        ));

        debug_assert!(unordered_elements_are(
            q.query((2, 2), (2, 2)),
            vec![(&((2, 2), (2, 2)), &10), (&((3, 3), (2, 2)), &55)]
        ));
    }

    #[test]
    fn query_mut_empty() {
        let mut q = Quadtree::<u32, u8>::new(2);
        let mut iter = q.query_mut((0, 0), (4, 4));
        debug_assert_eq!(iter.next(), None);
    }

    #[test]
    fn query_mut() {
        let mut q = Quadtree::<u32, u8>::new(3);

        // Insert #49 at (0, 0)->1x1.
        q.insert((0, 0), (1, 1), 49);
        // Up it to 50,
        for (_, i) in q.query_mut((0, 0), (1, 1)) {
            *i += 1;
        }
        // And verify.
        let mut tmp_iter_1 = q.query((0, 0), (1, 1));
        debug_assert_eq!(tmp_iter_1.size_hint(), (0, Some(1)));
        debug_assert_eq!(tmp_iter_1.next(), Some((&((0, 0), (1, 1)), &50)));
        debug_assert_eq!(tmp_iter_1.size_hint(), (0, Some(0)));
        debug_assert_eq!(tmp_iter_1.next(), None);
        debug_assert_eq!(tmp_iter_1.size_hint(), (0, Some(0)));

        // Insert #17 at (2, 2)->3x3.
        debug_assert!(q.insert((2, 2), (3, 3), 17));
        // Up it to 18,
        for (_, i) in q.query_mut((1, 1), (2, 2)) {
            *i += 1;
        }
        // And verify.
        let mut tmp_iter_2 = q.query((2, 2), (1, 1));
        debug_assert_eq!(tmp_iter_2.size_hint(), (0, Some(2)));
        debug_assert_eq!(tmp_iter_2.next(), Some((&((2, 2), (3, 3)), &18)));
        debug_assert_eq!(tmp_iter_2.size_hint(), (0, Some(1)));
        debug_assert_eq!(tmp_iter_2.next(), None);
        debug_assert_eq!(tmp_iter_2.size_hint(), (0, Some(0)));

        // Reset everything in (0, 0)->6x6 to "0".
        for (_, i) in q.query_mut((0, 0), (6, 6)) {
            *i = 0;
        }
        // Every value is now 0.

        for (_, v) in q.query((0, 0), (6, 6)) {
            debug_assert_eq!(*v, 0);
        }
    }

    #[test]
    fn query_pt_mut() {
        let mut q = Quadtree::<u32, u8>::new(4);
        // Insert #27 at (0, 0)->1x1.
        debug_assert!(q.insert((0, 0), (1, 1), 27));

        let mut tmp_iter = q.query_pt_mut((0, 0));
        debug_assert_eq!(tmp_iter.next(), Some((&((0, 0), (1, 1)), &mut 27)));
        debug_assert_eq!(tmp_iter.next(), None);
    }
}
