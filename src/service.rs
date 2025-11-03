#![allow(dead_code)]

use std::collections::HashMap;

use axum::http::{HeaderMap, HeaderValue};
use reqwest::Client;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
use rust_decimal::Decimal;

use crate::request::{
    MoralisTokenPriceInfo, MoralisTokenBalanceResponse,
};

const USER_AGENT: &str = "token-app/1.0";

const BASE_URL: &str = "https://deep-index.moralis.io/api/v2.2";
const MORALIS_API_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJub25jZSI6IjM5NDIzZTg5LTYyMjctNDhhNC04YmUyLWE5NzNhMDlmNzJkMyIsIm9yZ0lkIjoiNDc5MzE2IiwidXNlcklkIjoiNDkzMTE4IiwidHlwZUlkIjoiMGMxYzU5ODUtNmIwMy00Y2JhLTliMTYtZmJkYjhhNmJlMDhkIiwidHlwZSI6IlBST0pFQ1QiLCJpYXQiOjE3NjIxNzg2MjEsImV4cCI6NDkxNzkzODYyMX0.ilIjc2aqu0aY98w8jYClUQQ5VI2kYVvLRQDmdQZ6A-o";

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetBalanceRequest {
    pub address: String,
    pub contract_address: String,
}

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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SwapTokenResponse {}

#[derive(Debug, Clone)]
pub struct TokenService {
    tool_router: ToolRouter<Self>,
    client: reqwest::Client,
}

#[tool_router]
impl TokenService {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to create HTTP client");
        Self {
            client: client,
            tool_router: Self::tool_router(),
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
        Parameters(SwapTokenRequest { from_token, to_token, amount,slippage }): Parameters<SwapTokenRequest>,
    ) -> String {
        "ok".to_string()
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
        headers.insert("X-API-Key",HeaderValue::from_str(MORALIS_API_KEY).unwrap());

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
        let mut url = format!(
            "{}/wallets/{}/tokens?chain={}",
            BASE_URL,
            address,
            "eth",
        );

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
            instructions: Some("A simple calculator".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
