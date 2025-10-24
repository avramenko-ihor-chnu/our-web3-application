use application::{ExchangePrices, PolymarketSolana260};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ServerState {
    pub exchange_prices: Arc<RwLock<ExchangePrices>>,
    pub polymarket_solana260: Arc<RwLock<PolymarketSolana260>>,
}

#[derive(Debug, Deserialize)]
pub struct ActivePolymarketSearch {
    pub money: f64,
}
