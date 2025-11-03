//! # Application Server
//! Веб серверний компоннент, для застосунку _HedgeYourFun_
//! У обов'яки коптонетну входить:
//! - обслуговування користувача
//!   + надсилання сторінки, що ініцює взаємодіє з користувачем
//!   + обробка запитів зі сторінки користувача
//!   + частина реактивного інтерфейсу користувача
//! - синхронізацізалія данних використовуючи підєднані api
//! ## HTMX
//! Застосунок покладається на [htmx](https://htmx.org/) для UI
//! htmx-застосунки покладаються на сервер, що відпаравляє html
//! Htmx буде вкладати відповідь у сторінку для досягнення SPA UX
//! Наразі, застосунок відправляє як статичний (той, що не залежить від запиту користувача) html
//! Так і динамічний (той, що залежить від запиту користувача)
//! Побудова динамічних сторінок виконується шаблонним рушієм [askama](https://askama.readthedocs.io/en/stable/)
//! Виклик шаблонів можна впізнати за `templates::`, що є початком виклику одного зі шаблонів у `crates/server/src/templates.rs`
//! Виклик статичного html можна впізнати за `fs::read_to_string()`.
//! ## Tokio
//! `fs::read_to_string()` є скороченням від `tokio::fs::read_to_string`
//! `tokio` - асинхронний рантайм для `rust`
//! Ця програма асинхронна, `tokio` містить асинхронні api для стандартних функцій
//! Тому якщо у вас колись буде вибір між використанням `std::XXX` та `tokio::XXX`
//! у більшості випадків варто надати перевагу останньому.
//! ## Axum
//! `axum` це бекенд фреймворк.
//! Для озайомлення з його можливосятями відівідайте [https://github.com/tokio-rs/axum/tree/main/examples](https://github.com/tokio-rs/axum/tree/main/examples)
//! Оберіть бажану фічу та ознайомтеся з імлементацією
//! ## Cookbook
//! Іструкції для додавння функціоналу до цієї програми
//! ### Ви хочете додати новий шаблон та ендпоінт для нього
//! Для створення шаблону вам варто:
//! - створити `html` файл у `crates/server/templates/`
//! - створити `struct` (об'єкт) у `crates/server/src/templates.rs`
//! - ендпоінт, що буде відповідати на запит
//! - якщо, ви використовуєте html форму для отримання вводу користувача - поперньо створіть `struct` що відображає зміст цієї форми
//! ___
//! Ми хочемо взяти у користувача слово (`word`) та кількість (`number`), ми повернемо користувачу `ul>li{${rev_word}}*${number}`, де `rev_word` це слово задом-наперед.
//! Спершу варто всяти у користувача ввід. Додаймо `html` форму до нашого `index.html`:
//! ```html
//! <form hx-post="/word_number">
//!   <input name="word" type="text">
//!   <input name="number" type="number" min="0" step="1">
//!   <button type="submit">Submit</button>
//! </form>
//! ```
//! Тепер варто додати `struct` до `server.rs`, що відображає форму
//!
//! ```
//! #[derive(Debug, Deserialize)]
//! pub struct WordNumber {
//!     pub word: String,
//!     pub number: u32,
//! }
//! ```
//! При визначенні форми ми вказали намір роботи `POST` запити на `/word_number`
//! Додаймо цей ендпоінт до нашого роутера
//! ```
//! let app = Router::new()
//!     .route("/", get(index))
//!     .route("/word_number", post(word_number)) // <== наш новий ендпоінт
//!     .with_state(server_state);
//! ```
//! Перед тим як створити функцію `word_number`, створімо `html` шаблон, у `templates`
//! нехай це буде `templates/word_number.html`
//! ```jinja
//! <ul>
//!   {% for _ in range(end=number) %}
//!     <li>{{ rev_word }}</li>
//!   {% endfor %}
//! </ul>
//! ```
//! Додамо обробник шаблона до файлу `src/templates.rs`,
//! це `struct` який відоражає ввід для нашого шаблона,
//! ```
//! #[derive(Template)]
//! #[template(path = "word_number.html")]
//! pub struct WordNumber {
//!     pub rev_word: String,
//!     pub number: u32,
//! }
//! ```
//! `<'a>` поруч з `WordNumber` та `rev_word` означає, що `rev_word` мусить померти (бути стертим з пам'яті) по смерті `WordNumber`
//! Додамо обробника ендпоінта, раніше ми вказали, що цим буде займатися `word_number`
//! ```
//! async fn word_number(Form(WordNumber { word, number })) -> Result<Html<String>,StatusCode> {
//!     let rev_word: String = word.chars().rev().collect();
//!     let word_number = templates::WordNumber { rev_word, number };
//!     let html = word_number
//!         .render()
//!         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
//!
//!     Ok(Html(html))
//! }
//! ```
mod server;
mod templates;

use crate::server::{ActivePolymarketSearch, LoadAccount, ServerState};
use application::{AssetsRow, WalletService};
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

/// ## main
/// Це роутер, тут визначаються енд поінти які може обробити сервер.
/// ### route
/// Для додавання нового ендпоінта
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
    let wallet_service = WalletService::new();
    let exchange_rates = exchange_prices.read().await;

    let wallet_assets = wallet_service
        .get_wallet_assets(&account_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let assets_rows: Vec<AssetsRow> = wallet_assets
        .into_iter()
        .map(|asset| {
            let usd_value = if let Some(price) = exchange_rates.get_price(&asset.asset) {
                let balance: f64 = asset.balance.parse().unwrap_or(0.0); // Use asset.balance instead of asset.amount
                format!("${:.2}", balance * price)
            } else {
                "N/A".to_string()
            };

            AssetsRow {
                asset: asset.asset,
                balance: asset.balance,
                value: usd_value,
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
