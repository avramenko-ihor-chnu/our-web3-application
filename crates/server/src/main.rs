use std::{sync::Arc, time::Duration};

use application::{ExchangePrices, LamportBalance};
use askama::Template;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    routing::get,
};
use tokio::{fs, sync::RwLock};
use tower_http::services::ServeDir;
mod templates;

type SharedExchangePrices = Arc<RwLock<ExchangePrices>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shared_prices = start_price_updater();

    let app = Router::new()
        .route("/", get(index))
        .route("/hello-world", get(hello_world))
        .route("/favicon.ico", get(favicon))
        .route("/crypto-top", get(crypto_top))
        .route("/account/{id}", get(account))
        .nest_service("/css", ServeDir::new("crates/server/static/css"))
        .nest_service("/images", ServeDir::new("crates/server/static/images"))
        .nest_service("/effects", ServeDir::new("crates/server/static/effects"))
        .nest_service("/static", ServeDir::new("crates/server/static"))
        .with_state(shared_prices);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await?;
    println!("Running on http://0.0.0.0:8081");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn account(
    State(prices): State<SharedExchangePrices>,
    Path(id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let rate = prices.read().await.sol_to_usd;
    let lamport_balance = LamportBalance::get(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sol = lamport_balance.to_sol();
    let usd = lamport_balance.to_usd(rate);

    let exchange_prices = templates::ExchangeRate { sol, usd, rate };
    let html = exchange_prices
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

async fn hello_world() -> Html<String> {
    std::thread::sleep(std::time::Duration::from_secs(1)); // 😪
    Html("🤗Hello World!".to_string())
}

async fn favicon() -> axum::response::Redirect {
    axum::response::Redirect::permanent("/static/svg/icon.svg")
}

async fn crypto_top() -> Result<Html<String>, StatusCode> {
    let html = r#"
        <h2>Топ 5 криптовалют</h2>
        <table>
            <tr><td>1</td><td>Bitcoin</td><td>60000</td></tr>
            <tr><td>2</td><td>Ethereum</td><td>4000</td></tr>
        </table>
    "#;

    Ok(Html(html.to_string()))
}

pub fn start_price_updater() -> SharedExchangePrices {
    let prices = Arc::new(RwLock::new(ExchangePrices::new()));
    let prices_clone = Arc::clone(&prices);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300));

        loop {
            interval.tick().await;

            match ExchangePrices::get_sol_price().await {
                Ok(new_price) => {
                    let mut guard = prices_clone.write().await;
                    guard.sol_to_usd = new_price;
                    guard.last_updated = std::time::SystemTime::now();
                    println!("Updated SOL price: ${}", new_price);
                }
                Err(e) => eprintln!("Failed to fetch SOL price: {:?}", e),
            }
        }
    });

    prices
}
