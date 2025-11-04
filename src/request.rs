#[derive(Debug, serde::Deserialize)]
pub struct MoralisTokenPriceInfo {
    pub usdPriceFormatted: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct MoralisTokenBalanceInfo {
    pub symbol: String,
    pub balance_formatted: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct MoralisTokenBalanceResponse {
    pub result: Vec<MoralisTokenBalanceInfo>,
}

#[derive(Debug, serde::Serialize)]
pub struct EthCallRequest {}

#[derive(Debug, serde::Serialize)]
pub struct EthCallParam {
    pub from: String,
    pub to: String,
    pub gas: String,
    #[serde(rename = "gasPrice")]
    pub gas_price: String,
    pub value: String,
    pub data: String,
}
