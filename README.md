# token-mcp


#### 1.dependencies
```
- anyhow
- axum
- ethers
- reqwest
- rmcp
- rust_decimal
- serde
- serde_json
- tokio
- toml
- tracing
- tracing-subscriber
```

#### 2.how to run
##### 2.1 stratup command
```
npx @modelcontextprotocol/inspector cargo run
```
##### 2.2 local env
> change `wallet` value in **config.toml** file


#### 3. function call result
##### 3.1 get_balance
- request
```json
{
  "address": "0x388C818CA8B9251b393131C08a736A67ccB19297",
  "contract_address": "0x6B175474E89094C44Da98b954EedeAC495271d0F"
}
```

- response
```json
{
  "results": [
    {
      "symbol": "DAI",
      "amount": "0.82028192"
    }
  ]
}
```

##### 3.2 get_token_price
- request
```json
{
  "symbol": "0x6B175474E89094C44Da98b954EedeAC495271d0F"
}
```

- response
```json
1.000092077650548489
```
##### 3.3 swap_tokens
- request
```json
{
  "amount": "100",
  "from_token": "0x6B175474E89094C44Da98b954EedeAC495271d0F",
  "slippage": "0.05",
  "to_token": "0x6B175474E89094C44Da98b954EedeAC495271d0F"
}
```

- response
```json
{
  "status": "failed"
}
```
