use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize)]
pub struct List<T> {
    pub items: Vec<T>,
}

impl<T, U: Into<T>> FromIterator<U> for List<T> {
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        Self {
            items: iter.into_iter().map(Into::into).collect(),
        }
    }
}

/// Lets a PATCH tell "field absent" (None) from "field set to null" (Some(None)).
pub fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(de).map(Some)
}
