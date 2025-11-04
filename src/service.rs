#![allow(dead_code)]

use ethers::core::types::transaction::eip2718::TypedTransaction;
use ethers::{prelude::*, utils::parse_units};
use rust_decimal::prelude::ToPrimitive;
use std::{cmp::Ordering, ops::Mul, sync::Arc};

use axum::http::{HeaderMap, HeaderValue};
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};

use crate::{
    config::Config,
    request::{MoralisTokenBalanceResponse, MoralisTokenPriceInfo},
    util::{compare_float_str, get_amount_out_v2},
};

const USER_AGENT: &str = "token-app/1.0";

const BASE_URL: &str = "https://deep-index.moralis.io/api/v2.2";
const MORALIS_API_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJub25jZSI6IjM5NDIzZTg5LTYyMjctNDhhNC04YmUyLWE5NzNhMDlmNzJkMyIsIm9yZ0lkIjoiNDc5MzE2IiwidXNlcklkIjoiNDkzMTE4IiwidHlwZUlkIjoiMGMxYzU5ODUtNmIwMy00Y2JhLTliMTYtZmJkYjhhNmJlMDhkIiwidHlwZSI6IlBST0pFQ1QiLCJpYXQiOjE3NjIxNzg2MjEsImV4cCI6NDkxNzkzODYyMX0.ilIjc2aqu0aY98w8jYClUQQ5VI2kYVvLRQDmdQZ6A-o";
const INFRUA_URL: &str = "https://mainnet.infura.io/v3/5cc4bf9905bc4fe286eb5f900199b07f";

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetBalanceRequest {
    pub address: String,
    pub contract_address: String,
}

abigen!(IUniswapV2Pair, "abi/IUniswapV2Pair.json");
abigen!(IUniswapV2Factory, "abi/IUniswapV2Factory.json");
abigen!(
    UniRouter,
    r#"[
        function swapExactTokensForTokens(uint amountIn,uint amountOutMin,address[] path,address to,uint deadline) external returns (uint[] amounts)
    ]"#
);

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct GetBalanceResponse {
    pub results: Vec<BalanceInfo>,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct BalanceInfo {
    pub symbol: String,
    pub amount: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetTokenPriceRequest {
    pub symbol: String,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct GetTokenPriceResponse {
    pub symbol: String,
    pub price: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SwapTokenRequest {
    pub from_token: String,
    pub to_token: String,
    pub amount: String,
    pub slippage: String,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct SwapTokenResponse {
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct TokenService {
    tool_router: ToolRouter<Self>,
    client: reqwest::Client,
    wallet: String,
}

#[tool_router]
impl TokenService {
    pub fn new(config: Config) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to create HTTP client");
        Self {
            client: client,
            tool_router: Self::tool_router(),
            wallet: config.wallet,
        }
    }

    #[tool(description = "get balance")]
    async fn get_balance(
        &self,
        Parameters(GetBalanceRequest {
            address,
            contract_address,
        }): Parameters<GetBalanceRequest>,
    ) -> String {
        let mut response = GetBalanceResponse {
            results: Vec::new(),
        };

        if let Ok(res) = self.get_token_balance_v2(&address, &contract_address).await {
            for item in res.result {
                response.results.push(BalanceInfo {
                    symbol: item.symbol,
                    amount: item.balance_formatted,
                });
            }
        };

        serde_json::to_string(&response).unwrap()
    }

    #[tool(description = "get token price")]
    async fn get_token_price(
        &self,
        Parameters(GetTokenPriceRequest { symbol }): Parameters<GetTokenPriceRequest>,
    ) -> String {
        match self.inner_get_token_price(&symbol).await {
            Ok(res) => res.usdPriceFormatted,
            Err(_) => "0.00".to_string(),
        }
    }

    #[tool(description = "swap tokens")]
    async fn swap_tokens(
        &self,
        Parameters(SwapTokenRequest {
            from_token,
            to_token,
            amount,
            slippage,
        }): Parameters<SwapTokenRequest>,
    ) -> String {
        let mut resp = SwapTokenResponse {
            status: "ok".to_string(),
        };
        // check token amount
        let from_resp = self
            .get_token_balance_v2(&self.wallet, &from_token)
            .await
            .unwrap_or_default();
        if from_resp.result.is_empty() {
            resp.status = "failed".to_string();

            return serde_json::to_string(&resp).unwrap();
        }

        let from_amount = from_resp.result.first();

        match from_amount {
            Some(val) => match compare_float_str(&val.balance_formatted, &amount) {
                Some(order) => {
                    if order != Ordering::Greater {
                        resp.status = "failed".to_string();
                        tracing::info!("amount not match");
                        return serde_json::to_string(&resp).unwrap();
                    }
                }
                None => {
                    resp.status = "failed".to_string();
                    tracing::info!("get wallet token amount failed");
                    return serde_json::to_string(&resp).unwrap();
                }
            },
            None => {
                resp.status = "failed".to_string();
                return serde_json::to_string(&resp).unwrap();
            }
        }

        match self.inner_swap_token(&from_token, &to_token, &amount, &slippage).await {
            Ok(val) => tracing::info!("success to call swap token with result {}", val),
            Err(_) => {
                resp.status = "failed".to_string();
                tracing::info!("failed to call swap token");
                return serde_json::to_string(&resp).unwrap();
            }
        }

        serde_json::to_string(&resp).unwrap()
    }

    async fn inner_swap_token(
        &self,
        from: &str,
        to: &str,
        amount: &str,
        slippage: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let infrua_client = Provider::<Http>::try_from(INFRUA_URL).unwrap();
        let infrua_client = Arc::new(infrua_client);
        let deadline = U256::from(172_000_000_0_u64);

        //  Factory 合约
        let factory_addr: H160 = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".parse()?;
        let factory = IUniswapV2Factory::new(factory_addr, infrua_client.clone());
        let token_a: H160 = from.parse()?;
        let token_b: H160 = to.parse()?;
        let signer: H160 = self.wallet.parse()?;
        let pair_addr = factory.get_pair(token_a, token_b).call().await?;

        // 读取reverse
        let pair = IUniswapV2Pair::new(pair_addr, infrua_client.clone());

        let (reserve_in, reserve_out, _) = pair.get_reserves().call().await?;
        let amount_in = parse_units(amount, 18)?;
        let amount_out = get_amount_out_v2(amount_in.into(), reserve_in.into(), reserve_out.into());

        let slippage_bps = slippage
            .parse::<f64>()
            .unwrap()
            .mul(10_000_f64)
            .to_u64()
            .unwrap();
        let amount_out_min = amount_out * (10_000_u64 - slippage_bps) / 10_000_u64;

        let router_contract = UniRouter::new(factory_addr, infrua_client.clone());
        let path = vec![token_a, token_b];
        let amounts = router_contract
            .swap_exact_tokens_for_tokens(amount_in.into(), amount_out_min, path, signer, deadline)
            .from(signer)
            .call()
            .await?;

        tracing::info!("swap token success, swapOut token number {:?}", amounts);
        Ok("Success".to_string())
    }

    async fn inner_get_token_price(&self, address: &str) -> Result<MoralisTokenPriceInfo, String> {
        let query_url =
            format!("{}/erc20/{}/price?chain={}", BASE_URL, address, "eth",).to_string();

        self.make_request::<MoralisTokenPriceInfo>(&query_url).await
    }

    async fn make_request<T>(&self, url: &str) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        tracing::info!("Making request to: {}", url);

        let mut headers = HeaderMap::new();

        headers.insert("accept", HeaderValue::from_str("application/json").unwrap());
        headers.insert("X-API-Key", HeaderValue::from_str(MORALIS_API_KEY).unwrap());

        let response = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        tracing::info!("Received response: {:?}", response);

        match response.status() {
            reqwest::StatusCode::OK => response
                .json::<T>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e)),
            status => Err(format!("Request failed with status: {}", status)),
        }
    }

    async fn get_token_balance_v2(
        &self,
        address: &str,
        contract_address: &str,
    ) -> Result<MoralisTokenBalanceResponse, String> {
        let mut url = format!("{}/wallets/{}/tokens?chain={}", BASE_URL, address, "eth",);

        if !contract_address.is_empty() {
            url = format!(
                "{}/wallets/{}/tokens?chain={}&token_addresses%5B0%5D={}",
                BASE_URL, address, "eth", contract_address,
            );
        }

        self.make_request::<MoralisTokenBalanceResponse>(&url).await
    }
}

#[tool_handler]
impl ServerHandler for TokenService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A simple token service".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
