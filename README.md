## Rusty Client for dealing with Eden Mempool Streaming services

#### Usage: 
```Rust
// Initialize client
let url = Url::parse(EDEN_WSS_URL).unwrap();
let client = Client::new(url);

// Subscribe to a stream
let mut stream = client.subscribe_txs().await.unwrap();

// Listen for new pedning txs
while let Some(tx) = stream.next().await {
    // types::EdenPendingTx
    dbg!(&tx);
    // ethers_core::types::Transaction
    dbg!(&tx.into_ethers_tx());
    ...
}
```
