use crate::geometry::area::Area;
use std::collections::HashMap;
use uuid::Uuid;

pub(crate) type StoreType<U, V> = HashMap<Uuid, (Area<U>, V)>;
