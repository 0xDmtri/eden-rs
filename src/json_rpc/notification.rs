use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Serialize,
};

use crate::json_rpc::response::{Response, ResponsePayload};
use crate::types::EdenPendingTx;

#[derive(Debug, Clone)]
pub enum EdenItem {
    Response(Response),
    Notification(EdenNotification),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EdenNotification {
    pub subscription: u64,
    pub result: EdenPendingTx,
}

impl<'de> Deserialize<'de> for EdenItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EdenItemVisitor;

        impl<'de> Visitor<'de> for EdenItemVisitor {
            type Value = EdenItem;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a JSON-RPC response or an Ethereum-style notification")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut id = None;
                let mut result = None;
                let mut params = None;
                let mut error = None;

                // Drain the map into the appropriate fields
                while let Ok(Some(key)) = map.next_key() {
                    match key {
                        "id" => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        "result" => {
                            if result.is_some() {
                                return Err(serde::de::Error::duplicate_field("result"));
                            }
                            result = Some(map.next_value()?);
                        }
                        "params" => {
                            if params.is_some() {
                                return Err(serde::de::Error::duplicate_field("params"));
                            }
                            params = Some(map.next_value()?);
                        }
                        "error" => {
                            if error.is_some() {
                                return Err(serde::de::Error::duplicate_field("error"));
                            }
                            error = Some(map.next_value()?);
                        }
                        // Discard unknown fields
                        _ => {
                            let _ = map.next_value::<serde_json::Value>()?;
                        }
                    }
                }

                // If it has an ID, it is a response
                if let Some(id) = id {
                    let payload = error
                        .map(ResponsePayload::Failure)
                        .or_else(|| result.map(ResponsePayload::Success))
                        .ok_or_else(|| {
                            serde::de::Error::custom(
                                "missing `result` or `error` field in response",
                            )
                        })?;

                    Ok(EdenItem::Response(Response { id, payload }))
                } else {
                    if error.is_some() {
                        return Err(serde::de::Error::custom(
                            "unexpected `error` field in subscription notification",
                        ));
                    }
                    params
                        .map(EdenItem::Notification)
                        .ok_or_else(|| serde::de::Error::missing_field("params"))
                }
            }
        }

        deserializer.deserialize_any(EdenItemVisitor)
    }
}

#[cfg(test)]
mod tests {
    use ethers_core::types::U256;
    use eyre::Result;

    use super::*;

    use crate::json_rpc::id::Id;

    #[test]
    fn deser_notification_test() -> Result<()> {
        // EDEN URL
        let notification = r#"{"jsonrpc":"2.0","method":"subscription","params":{"subscription":4815270595554998,"result":{"type":"0x2","hash":"0xd2bd5a7fa523f13e7f955c0753cd2f1de0635b6c165c2494aae44d8bbdd9a9c6","from":"0x19450678803d6a7bb6897ca1e793a071a100cba7","nonce":"0x2","gasLimit":"0x7a120","to":"0x19c10fff96b80208f454034c046ccc4445cd20ba","data":"0x886f9ece000000000000000000000000000000000000000000000000083019dfc17b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000659f3fdb00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000041c63f9a4c2d53866c5a88bd5dfceab7c4ac0733b1d2b788ec9293bbaffc8f031b1ce884faad136a6a9dca6b60ccab9f13d82c492c7414b0a66a518c7a36f8ade01b00000000000000000000000000000000000000000000000000000000000000","v":"0x26","r":"0xe6e52e08bf9735e38c1808285269afef6b82d500cd5a90966479b5f8fa70e623","s":"0x21490c9a52a60b2c3a5a6045d687dbe8a5e710274aa3071b813a1bf24271eb45","value":"0x83019dfc17b0000","chainId":"0x1","accessList":[],"maxPriorityFeePerGas":"0x2faf080","maxFeePerGas":"0xc570bd200"}}}"#;

        let deser = serde_json::from_str::<EdenItem>(notification)?;

        match deser {
            EdenItem::Notification(EdenNotification {
                subscription,
                result,
            }) => {
                assert_eq!(subscription, 4815270595554998);
                assert_eq!(result.nonce, U256::from(2));
            }
            _ => panic!("unexpected deserialization result"),
        }

        Ok(())
    }

    #[test]
    fn deser_response_test() -> Result<()> {
        // EDEN URL
        let response = r#"{"jsonrpc":"2.0","result":4815270595554998,"id":1}"#;

        let deser = serde_json::from_str::<EdenItem>(response)?;

        match deser {
            EdenItem::Response(Response { id, payload }) => {
                assert_eq!(id, Id::Number(1));
                assert!(payload.is_success());
            }
            _ => panic!("unexpected deserialization result"),
        }

        Ok(())
    }

    #[test]
    fn deser_error_test() -> Result<()> {
        // EDEN URL
        let error =
            r#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error"},"id":null}"#;

        let deser = serde_json::from_str::<EdenItem>(error)?;

        match deser {
            EdenItem::Response(Response { id: _, payload }) => {
                assert!(payload.is_error());
            }
            _ => panic!("unexpected deserialization result"),
        }

        Ok(())
    }
}
