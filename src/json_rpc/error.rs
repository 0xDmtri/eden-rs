use core::marker::PhantomData;

use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Serialize,
};
use serde_json::value::RawValue;

/// A JSONRPC-2.0 error object.
///
/// This response indicates that the server received and handled the request,
/// but that there was an error in the processing of it. The error should be
/// included in the `message` field of the response payload.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorPayload<ErrData = Box<RawValue>> {
    /// The error code.
    pub code: i64,
    /// The error message (if any).
    pub message: String,
    /// The error data (if any).
    pub data: Option<ErrData>,
}

impl<ErrData> std::fmt::Display for ErrorPayload<ErrData> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ErrorPayload code {}, message: \"{}\", contains payload: {}",
            self.code,
            self.message,
            self.data.is_some()
        )
    }
}

impl<'de, ErrData: Deserialize<'de>> Deserialize<'de> for ErrorPayload<ErrData> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        enum Field {
            Code,
            Message,
            Data,
            Unknown,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> serde::de::Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        formatter.write_str("`code`, `message` and `data`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "code" => Ok(Field::Code),
                            "message" => Ok(Field::Message),
                            "data" => Ok(Field::Data),
                            _ => Ok(Field::Unknown),
                        }
                    }
                }
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ErrorPayloadVisitor<T>(PhantomData<T>);

        impl<'de, Data> Visitor<'de> for ErrorPayloadVisitor<Data>
        where
            Data: Deserialize<'de>,
        {
            type Value = ErrorPayload<Data>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a JSON-RPC2.0 error object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut code = None;
                let mut message = None;
                let mut data = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Code => {
                            if code.is_some() {
                                return Err(serde::de::Error::duplicate_field("code"));
                            }
                            code = Some(map.next_value()?);
                        }
                        Field::Message => {
                            if message.is_some() {
                                return Err(serde::de::Error::duplicate_field("message"));
                            }
                            message = Some(map.next_value()?);
                        }
                        Field::Data => {
                            if data.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data = Some(map.next_value()?);
                        }
                        Field::Unknown => {
                            // ignore
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                Ok(ErrorPayload {
                    code: code.ok_or_else(|| serde::de::Error::missing_field("code"))?,
                    message: message.unwrap_or_default(),
                    data,
                })
            }
        }

        deserializer.deserialize_any(ErrorPayloadVisitor(PhantomData))
    }
}
