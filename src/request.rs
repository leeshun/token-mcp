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
