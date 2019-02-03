use {crate::entry::Entry, std::collections::HashMap, uuid::Uuid};

pub(crate) type StoreType<U, V> = HashMap<Uuid, Entry<U, V>>;
