use serde::Serialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Debug)]
pub enum AppError {
    InvalidWalletAddress(String),
    ErrorFetchingBalance,
    ExchangePriceApiErr,
    PolymarketApiErr,
}

#[derive(Debug)]
pub struct TradeCalculation {
    pub estimated_cost: f64,
    pub price_per_share: f64,
    pub shares: usize,
    pub total_cost: f64,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::InvalidWalletAddress(e) => write!(f, "Invalid Wallet Address: {}", e),
            AppError::ErrorFetchingBalance => write!(f, "Error fetching balance"),
            AppError::ExchangePriceApiErr => todo!(),
            AppError::PolymarketApiErr => todo!(),
        }
    }
}

impl std::error::Error for AppError {}

pub struct LamportBalance(u64);

#[derive(Debug, Clone, Serialize)]
pub struct ExchangePrices {
    pub last_updated: std::time::SystemTime,
    pub sol_to_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolymarketSolana260 {
    pub last_updated: std::time::SystemTime,
    pub answer_no_multiplier: f64,
}

impl PolymarketSolana260 {
    pub fn new() -> Self {
        Self {
            last_updated: std::time::SystemTime::UNIX_EPOCH,
            answer_no_multiplier: 0.0,
        }
    }

    pub async fn update() -> Result<f64, AppError> {
        let url = "https://gamma-api.polymarket.com/markets/slug/will-solana-reach-260-before-2026-327-264-879-598";
        let response = reqwest::get(url)
            .await
            .map_err(|_| AppError::PolymarketApiErr)?;
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|_| AppError::PolymarketApiErr)?;

        let outcome_prices = json["outcomePrices"]
            .as_str()
            .ok_or(AppError::PolymarketApiErr)?;

        let prices_value: serde_json::Value =
            serde_json::from_str(outcome_prices).map_err(|_| AppError::PolymarketApiErr)?;

        let raw_prices: Vec<f64> = prices_value
            .as_array()
            .ok_or(AppError::PolymarketApiErr)?
            .iter()
            .map(|v| v.as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0))
            .collect();

        const POLYMARKET_FEE: f64 = 0.02;

        let website_prices: Vec<f64> = raw_prices
            .iter()
            .map(|&raw_price| raw_price + POLYMARKET_FEE)
            .collect();

        println!("Raw prices: {:?}", raw_prices);
        println!("Website prices: {:?}", website_prices);

        Ok(website_prices[1])
    }
}

impl ExchangePrices {
    pub fn new() -> Self {
        Self {
            last_updated: std::time::SystemTime::UNIX_EPOCH,
            sol_to_usd: 0.0,
        }
    }

    pub async fn get_sol_price() -> Result<f64, AppError> {
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd";
        let response = reqwest::get(url)
            .await
            .map_err(|_| AppError::ExchangePriceApiErr)?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|_| AppError::ExchangePriceApiErr)?;

        json["solana"]["usd"]
            .as_f64()
            .ok_or(AppError::ExchangePriceApiErr)
    }

    pub fn get_sol_to_usd(&self) -> f64 {
        self.sol_to_usd
    }

    pub fn get_last_updated(&self) -> std::time::SystemTime {
        self.last_updated
    }
}

impl LamportBalance {
    pub fn to_usd(&self, sol_to_usd: f64) -> f64 {
        let self_sol = self.to_sol();
        self_sol * sol_to_usd
    }
    pub fn to_sol(&self) -> f64 {
        self.0 as f64 / 1_000_000_000.0
    }
    pub async fn get(wallet_address: String) -> Result<Self, AppError> {
        let pubkey = Pubkey::from_str(&wallet_address)
            .map_err(|_| AppError::InvalidWalletAddress(wallet_address))?;

        let rpc_url = "https://api.devnet.solana.com".to_string();
        let client = RpcClient::new(rpc_url);
        let balance = client
            .get_balance(&pubkey)
            .map_err(|_| AppError::ErrorFetchingBalance)?;
        Ok(LamportBalance(balance))
    }
}
