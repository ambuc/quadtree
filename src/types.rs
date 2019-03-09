use {crate::entry::Entry, std::collections::HashMap};

pub(crate) type StoreType<U, V> = HashMap<u64, Entry<U, V>>;
