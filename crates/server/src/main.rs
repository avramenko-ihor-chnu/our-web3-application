mod server;
mod templates;

use crate::{
    server::{ActivePolymarketSearch, LoadAccount, ServerState},
    templates::AssetsRow,
};
use application::{ExchangePrices, LamportBalance, PolymarketSolana260};

use askama::Template;
use axum::{
    Form, Router,
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
};
use std::{sync::Arc, time::Duration};
use tokio::{fs, sync::RwLock};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_state = server_state_updater();

    let app = Router::new()
        .route("/", get(index))
        .route("/account", post(account))
        .route("/positions", post(positions))
        .route("/favicon.ico", get(favicon))
        .route("/calculator", get(calculator_body))
        .route("/calculator", post(calc))
        .nest_service("/css", ServeDir::new("crates/server/static/css"))
        .nest_service("/js", ServeDir::new("crates/server/static/js"))
        .nest_service("/images", ServeDir::new("crates/server/static/images"))
        .nest_service("/effects", ServeDir::new("crates/server/static/effects"))
        .nest_service("/static", ServeDir::new("crates/server/static"))
        .with_state(server_state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8081".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    println!("Running on http://0.0.0.0:{port}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn calculator_body() -> Result<Html<String>, StatusCode> {
    match tokio::fs::read_to_string("crates/server/templates/calculator.html").await {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn calc(
    State(ServerState {
        exchange_prices: _,
        polymarket_solana260,
    }): State<ServerState>,
    Form(ActivePolymarketSearch { money }): Form<ActivePolymarketSearch>,
) -> Result<Html<String>, StatusCode> {
    let polymarket_solana260_multiplyer = polymarket_solana260.read().await.answer_no_multiplier;
    let bet_return = money / polymarket_solana260_multiplyer;
    let html = format!("{bet_return:.2}$");
    Ok(Html(html))
}

async fn account(
    State(ServerState {
        exchange_prices,
        polymarket_solana260: _,
    }): State<ServerState>,
    Form(LoadAccount { account_id }): Form<LoadAccount>,
) -> Result<Html<String>, StatusCode> {
    let rate = exchange_prices.read().await.sol_to_usd;
    let lamport_balance = LamportBalance::get(account_id)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let sol = &format!("{:.2}", lamport_balance.to_sol());
    let usd = &format!("{:.2}", lamport_balance.to_usd(rate));
    let rate = &format!("{:.2}", exchange_prices.read().await.sol_to_usd);

    let exchange_prices = templates::ExchangeRate { sol, usd, rate };
    let html = exchange_prices
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

async fn positions(
    State(ServerState {
        exchange_prices,
        polymarket_solana260: _,
    }): State<ServerState>,
    Form(LoadAccount { account_id }): Form<LoadAccount>,
) -> Result<Html<String>, StatusCode> {
    let rate = exchange_prices.read().await.sol_to_usd;
    let lamport_balance = LamportBalance::get(account_id)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let sol = lamport_balance.to_sol();
    let usd = lamport_balance.to_usd(rate);
    // Активи користувача (затичка / треба замінити на реальні підтягування)
    let assets = vec![
        ("SOL", sol, usd),
        ("USDC", 150.0, 150.0),
        ("BONK", 1200000.0, 30.0),
    ];
    let assets_rows: Vec<AssetsRow> = assets
        .iter()
        .map(|(asset, balance, value)| {
            let asset = asset.to_string();
            let balance = format!("{balance:.2}");
            let value = format!("{value:.2}");
            AssetsRow {
                asset,
                balance,
                value,
            }
        })
        .collect();
    let html = templates::AccountAssets { assets_rows }
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(html))
}

async fn index() -> Result<Html<String>, StatusCode> {
    let index = fs::read_to_string("crates/server/templates/index.html")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(index))
}

async fn favicon() -> axum::response::Redirect {
    axum::response::Redirect::permanent("/static/svg/icon.svg")
}

fn server_state_updater() -> ServerState {
    let exchange_prices = Arc::new(RwLock::new(ExchangePrices::new()));
    let polymarket_solana260 = Arc::new(RwLock::new(PolymarketSolana260::new()));

    let exchange_prices_clone = Arc::clone(&exchange_prices);
    let polymarket_solana260_clone = Arc::clone(&polymarket_solana260);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            match ExchangePrices::get_sol_price().await {
                Ok(new_exchange_prices) => {
                    let mut guard = exchange_prices_clone.write().await;
                    guard.last_updated = std::time::SystemTime::now();
                    guard.sol_to_usd = new_exchange_prices;
                }
                _ => continue,
            }
        }
    });
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            match PolymarketSolana260::update().await {
                Ok(new_polymarket_solana260) => {
                    let mut guard = polymarket_solana260_clone.write().await;
                    guard.last_updated = std::time::SystemTime::now();
                    guard.answer_no_multiplier = new_polymarket_solana260;
                }
                _ => continue,
            }
        }
    });

    ServerState {
        exchange_prices,
        polymarket_solana260,
    }
}
