use {crate::entry::Entry, std::collections::HashMap};

// d888888b db    db d8888b. d88888b .d8888.
// `~~88~~' `8b  d8' 88  `8D 88'     88'  YP
//    88     `8bd8'  88oodD' 88ooooo `8bo.
//    88       88    88~~~   88~~~~~   `Y8b.
//    88       88    88      88.     db   8D
//    YP       YP    88      Y88888P `8888Y'

pub(crate) type StoreType<U, V> = HashMap<u64, Entry<U, V>>;
