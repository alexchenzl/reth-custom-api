# reth-custom-api
An example to extend Reth RPC api
In this crate, we add a new API `getAccountExt` to get account balance, nonce and code hash in one RPC call.

# Run
```
cargo run -- node --http --http.api=eth --eth-ext
```
# Query
```
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_getAccountExt","params":["0xADDRESS", "latest"],"id":1}' http://localhost:8545 | jq
```