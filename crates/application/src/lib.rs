use axum::http::StatusCode;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;

pub struct AssetsRow {
    pub asset: String,
    pub balance: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
struct ZerionPortfolioResponse {
    links: ZerionLinks,
    data: ZerionPortfolioData,
}

#[derive(Debug, Deserialize)]
struct ZerionLinks {
    #[serde(rename = "self")]
    self_link: String,
}

#[derive(Debug, Deserialize)]
struct ZerionPortfolioData {
    #[serde(rename = "type")]
    data_type: String,
    id: String,
    attributes: ZerionAttributes,
}

#[derive(Debug, Deserialize)]
struct ZerionAttributes {
    positions_distribution_by_type: HashMap<String, f64>,
    positions_distribution_by_chain: HashMap<String, f64>,
    total: ZerionTotal,
    changes: ZerionChanges,
}

#[derive(Debug, Deserialize)]
struct ZerionTotal {
    positions: f64,
}

#[derive(Debug, Deserialize)]
struct ZerionChanges {
    absolute_1d: f64,
    percent_1d: Option<f64>,
}

struct ZerionClient {
    client: Client,
    auth_header: String,
}

impl ZerionClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth_header: "Basic emtfZGV2X2RhNzg4OWRmMjUwZTQ1ZTFhNzAwY2M3OTg1YjE2MTQ3Og=="
                .to_string(),
        }
    }

    pub async fn get_portfolio(&self, address: &str) -> Result<ZerionPortfolioResponse, AppError> {
        let url = format!(
            "https://api.zerion.io/v1/wallets/{}/portfolio?currency=usd",
            address
        );

        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .header("authorization", &self.auth_header)
            .send()
            .await
            .map_err(|_| AppError::ZerionApiErr)?;

        if !response.status().is_success() {
            return Err(AppError::ZerionApiErr);
        }

        let portfolio_data = response
            .json::<ZerionPortfolioResponse>()
            .await
            .map_err(|_| AppError::ZerionApiErr)?;

        Ok(portfolio_data)
    }
}

#[derive(Debug)]
pub enum AppError {
    InvalidWalletAddress(String),
    ErrorFetchingBalance,
    ExchangePriceApiErr,
    PolymarketApiErr,
    ZerionApiErr,
    SolanaRpcErr,
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
            AppError::ZerionApiErr => todo!(),
            AppError::SolanaRpcErr => todo!(),
        }
    }
}

impl Into<StatusCode> for AppError {
    fn into(self) -> StatusCode {
        match self {
            AppError::PolymarketApiErr => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ZerionApiErr => StatusCode::BAD_REQUEST,
            AppError::SolanaRpcErr => StatusCode::BAD_REQUEST,
            AppError::InvalidWalletAddress(_) => StatusCode::BAD_REQUEST,
            AppError::ErrorFetchingBalance => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ExchangePriceApiErr => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub struct WalletService {
    zerion_client: ZerionClient,
    // Keep solana client if you still need it for specific Solana-only features
}

impl WalletService {
    pub fn new() -> Self {
        Self {
            zerion_client: ZerionClient::new(),
        }
    }

    pub async fn get_wallet_assets(&self, address: &str) -> Result<Vec<AssetsRow>, AppError> {
        // First try Zerion API for multi-chain support
        match self.zerion_client.get_portfolio(address).await {
            Ok(portfolio) => self.zerion_portfolio_to_assets(portfolio, address).await,
            Err(_) => {
                // If Zerion fails, fall back to Solana-only approach
                self.get_solana_assets(address).await
            }
        }
    }

    async fn zerion_portfolio_to_assets(
        &self,
        portfolio: ZerionPortfolioResponse,
        address: &str,
    ) -> Result<Vec<AssetsRow>, AppError> {
        let mut assets = Vec::new();

        // Check if portfolio is empty (likely invalid address or no assets)
        if portfolio.data.attributes.total.positions == 0.0 {
            // Try to get detailed positions to see if there are actually assets
            // or if this is an invalid address
            return self.get_detailed_positions(address).await;
        }

        // Process chain distribution
        for (chain, value) in portfolio.data.attributes.positions_distribution_by_chain {
            if value > 0.0 {
                assets.push(AssetsRow {
                    asset: chain,
                    balance: format!("{:.2}", value), // This is USD value
                    value: format!("{:.2}", value),
                });
            }
        }

        // If we have assets, return them
        if !assets.is_empty() {
            Ok(assets)
        } else {
            // Fall back to Solana if no assets found
            self.get_solana_assets(address).await
        }
    }

    async fn get_detailed_positions(&self, address: &str) -> Result<Vec<AssetsRow>, AppError> {
        // You can implement this later to get individual token positions
        // For now, fall back to Solana
        self.get_solana_assets(address).await
    }

    async fn get_solana_assets(&self, address: &str) -> Result<Vec<AssetsRow>, AppError> {
        // Your existing Solana balance logic here
        let lamport_balance = LamportBalance::get(address.to_string())
            .await
            .map_err(|_| AppError::SolanaRpcErr)?;

        let sol = lamport_balance.to_sol();
        // For USD conversion, you might need to get current SOL price
        let usd = sol * 100.0; // Placeholder - replace with actual rate

        let assets = vec![AssetsRow {
            asset: "SOL".to_string(),
            balance: format!("{:.6}", sol),
            value: format!("{:.2}", usd),
        }];

        Ok(assets)
    }
}

impl std::error::Error for AppError {}

pub struct LamportBalance(u64);

#[derive(Debug, Clone, Serialize)]
pub struct ExchangePrices {
    pub last_updated: std::time::SystemTime,
    pub sol_to_usd: f64,
    pub btc_to_usd: f64,
    pub eth_to_usd: f64,
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
        Ok(website_prices[1])
    }
}

impl ExchangePrices {
    pub fn new() -> Self {
        Self {
            last_updated: std::time::SystemTime::UNIX_EPOCH,
            sol_to_usd: 0.0,
            btc_to_usd: 0.0,
            eth_to_usd: 0.0,
        }
    }
    pub async fn update(&mut self) -> Result<(), AppError> {
        let sol_future = Self::get_sol_price();
        let btc_future = Self::get_btc_price();
        let eth_future = Self::get_eth_price();

        let (sol_price, btc_price, eth_price) =
            tokio::try_join!(sol_future, btc_future, eth_future)?;

        self.sol_to_usd = sol_price;
        self.btc_to_usd = btc_price;
        self.eth_to_usd = eth_price;
        self.last_updated = std::time::SystemTime::now();

        Ok(())
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

    pub async fn get_btc_price() -> Result<f64, AppError> {
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";
        let response = reqwest::get(url)
            .await
            .map_err(|_| AppError::ExchangePriceApiErr)?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|_| AppError::ExchangePriceApiErr)?;

        json["bitcoin"]["usd"]
            .as_f64()
            .ok_or(AppError::ExchangePriceApiErr)
    }

    pub async fn get_eth_price() -> Result<f64, AppError> {
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd";
        let response = reqwest::get(url)
            .await
            .map_err(|_| AppError::ExchangePriceApiErr)?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|_| AppError::ExchangePriceApiErr)?;

        json["ethereum"]["usd"]
            .as_f64()
            .ok_or(AppError::ExchangePriceApiErr)
    }

    pub fn get_price(&self, symbol: &str) -> Option<f64> {
        match symbol.to_lowercase().as_str() {
            "sol" | "solana" => Some(self.sol_to_usd),
            "btc" | "bitcoin" => Some(self.btc_to_usd),
            "eth" | "ethereum" => Some(self.eth_to_usd),
            _ => None,
        }
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
