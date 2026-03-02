// ==================================================
//! Serde helpers.

use log::error;
use serde::{Deserialize, Deserializer};

use crate::{LOG_TARGET_CONFIG, bindings::Binding};
pub(crate) struct StringVisitor;
impl serde::de::Visitor<'_> for StringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(s.to_lowercase())
    }
}

pub fn deserialize_bindings<'a, D, T>(
    deserializer: D,
    mut default: Vec<Binding<T>>,
) -> Result<Vec<Binding<T>>, D::Error>
where
    D: Deserializer<'a>,
    T: Clone + Eq,
    Binding<T>: Deserialize<'a>,
{
    let values = Vec::<toml::Value>::deserialize(deserializer)?;

    // Skip all invalid values.
    let mut bindings = Vec::with_capacity(values.len());
    for value in values {
        match Binding::<T>::deserialize(value) {
            Ok(binding) => bindings.push(binding),
            Err(err) => {
                error!(target: LOG_TARGET_CONFIG, "Config error: {err}; ignoring binding");
            },
        }
    }

    // Remove matching default bindings.
    for binding in bindings.iter() {
        default.retain(|b| !b.triggers_match(binding));
    }

    bindings.extend(default);

    Ok(bindings)
}
