use crate::entry::Entry;
use std::collections::HashMap;
use uuid::Uuid;

pub(crate) type StoreType<U, V> = HashMap<Uuid, Entry<U, V>>;
