use std::hash::Hash;

// Inspired by google/googletest's UnorderedElementsAre().
// https://github.com/google/googletest/blob/master/googlemock/docs/CheatSheet.md#container-matchers
//
// This function only gets used in tests.
#[allow(dead_code)]
pub fn unordered_elements_are<T, X, Y>(x: X, y: Y) -> bool
where
    X: IntoIterator<Item = T>,
    X::Item: PartialEq + Eq + Hash,
    Y: IntoIterator<Item = T>,
    Y::Item: PartialEq + Eq + Hash,
{
    use std::collections::HashSet;
    use std::iter::FromIterator;

    let hs1: HashSet<T> = HashSet::from_iter(x);
    let hs2: HashSet<T> = HashSet::from_iter(y);
    hs1 == hs2
}
