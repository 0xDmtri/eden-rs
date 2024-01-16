use eyre::Result;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_tungstenite::{
    connect_async, tungstenite::client::IntoClientRequest, tungstenite::Message, MaybeTlsStream,
    WebSocketStream,
};
use url::Url;

use crate::{json_rpc::notification::EdenItem, types::EdenPendingTx};

// declare type aliases
pub type TungsteniteStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type Writer = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

/// Eden Mempool Client
pub struct Client {
    pub(crate) url: Url,
}

impl Client {
    /// Initialize new client with eden agg mempool url
    pub fn new(wss: impl Into<Url>) -> Self {
        Self { url: wss.into() }
    }

    // sends pending tx subscription msg
    async fn subscribe_internal(stream: &mut Writer, params: &[&str]) -> Result<()> {
        let params = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "subscribe",
            "params": params,
        });

        let params_str = serde_json::to_string(&params)?;

        Ok(stream.send(Message::Text(params_str)).await?)
    }

    /// subscribes and returns stream of `EdenPedningTx`
    pub async fn subscribe_txs(&self) -> Result<UnboundedReceiverStream<EdenPendingTx>> {
        let req = self.url.clone().into_client_request()?;
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let (stream, _) = connect_async(req.clone()).await?;
            let (mut write, mut read) = stream.split();

            // subsctibe to full pednings txs
            Self::subscribe_internal(&mut write, &["newTxs"]).await?;

            // handle stream data
            while let Some(item) = read.next().await {
                match item {
                    Ok(payload) => match payload {
                        Message::Text(text) => {
                            // deserialize
                            let item: EdenItem = serde_json::from_str(&text)?;

                            // match if it is a `Notification` or `Response`
                            match item {
                                EdenItem::Response(r) => {
                                    if r.payload.is_error() {
                                        tracing::error!("Error in reponse: {:?}", r.payload);
                                    }
                                }
                                EdenItem::Notification(n) => {
                                    tx.send(n.result)?;
                                }
                            }
                        }
                        Message::Pong(pong_data) => {
                            tracing::debug!("Received Pong");
                            write.send(Message::Ping(pong_data)).await?;
                        }
                        Message::Ping(ping_data) => {
                            tracing::debug!("Received Ping");
                            write.send(Message::Pong(ping_data)).await?;
                        }
                        Message::Close(frame) => {
                            if frame.is_some() {
                                tracing::error!(?frame, "Received close frame with data");
                            } else {
                                tracing::error!("WS server has gone away");
                            }
                            return Err(eyre::eyre!("Stream has been closed"));
                        }
                        _ => {}
                    },
                    Err(e) => {
                        tracing::error!(error = ?e, "Error in transaction stream");
                        break;
                    }
                }
            }

            eyre::Ok(())
        });

        Ok(UnboundedReceiverStream::new(rx))
    }
}

#[cfg(test)]
mod tests {

    const MEMPOOL_WS: &str = "wss://speed-eu-west.edennetwork.io";

    use super::*;

    #[tokio::test]
    async fn test_txs_subscription() {
        let url = Url::parse(MEMPOOL_WS).unwrap();
        let client = Client::new(url);

        let mut stream = client.subscribe_txs().await.unwrap();
        let pedning_tx = stream.next().await;

        assert!(pedning_tx.is_some());
    }
}
