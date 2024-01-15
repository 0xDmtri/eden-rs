use std::fmt::Display;

use serde::{de::Visitor, Deserialize, Serialize};

/// A JSON-RPC 2.0 ID object. This may be a number, a string, or null.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Id {
    /// A number.
    Number(u64),
    /// A string.
    String(String),
    /// Null.
    None,
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Id::Number(n) => write!(f, "{}", n),
            Id::String(s) => write!(f, "{}", s),
            Id::None => write!(f, "null"),
        }
    }
}

impl Serialize for Id {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Id::Number(n) => serializer.serialize_u64(*n),
            Id::String(s) => serializer.serialize_str(s),
            Id::None => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct IdVisitor;

        impl<'de> Visitor<'de> for IdVisitor {
            type Value = Id;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a string, a number, or null")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Id::String(v.to_owned()))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Id::Number(v))
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Id::None)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Id::None)
            }
        }

        deserializer.deserialize_any(IdVisitor)
    }
}

impl Id {
    /// Returns `true` if the ID is a number.
    pub const fn is_number(&self) -> bool {
        matches!(self, Id::Number(_))
    }

    /// Returns `true` if the ID is a string.
    pub const fn is_string(&self) -> bool {
        matches!(self, Id::String(_))
    }

    /// Returns `true` if the ID is `None`.
    pub const fn is_none(&self) -> bool {
        matches!(self, Id::None)
    }

    /// Returns the ID as a number, if it is one.
    pub const fn as_number(&self) -> Option<u64> {
        match self {
            Id::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Returns the ID as a string, if it is one.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Id::String(s) => Some(s),
            _ => None,
        }
    }
}
