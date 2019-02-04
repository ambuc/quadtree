use {
    num::{cast::FromPrimitive, PrimInt},
    std::hash::Hash,
};

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
    use std::{collections::HashSet, iter::FromIterator};

    let hs1: HashSet<T> = HashSet::from_iter(x);
    let hs2: HashSet<T> = HashSet::from_iter(y);
    hs1 == hs2
}

#[allow(dead_code)]
pub fn print_quadtree<U, V>(qt: &quadtree_impl::Quadtree<U, V>)
where
    U: PrimInt + FromPrimitive + std::fmt::Debug,
    V: std::fmt::Debug,
{
    print!("┌");
    for _i in 0..qt.width() {
        print!("─");
    }
    print!("┐\n");
    for i in 0..qt.width() {
        print!("│");
        for j in 0..qt.height() {
            match qt
                .query(
                    (U::from_usize(i).unwrap(), U::from_usize(j).unwrap()),
                    (U::one(), U::one()),
                )
                .count()
            {
                0 => print!(" "),
                1 => print!("░"),
                2 => print!("▒"),
                3 => print!("▓"),
                _ => print!("█"),
            }
        }
        print!("|");
        print!("\n");
    }
    print!("└");
    for _i in 0..qt.width() {
        print!("─");
    }
    print!("┘\n");
}
