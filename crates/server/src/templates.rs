use askama::Template;
use serde::Serialize;

#[derive(Template)]
#[template(path = "exchange-rate.html")]
pub struct ExchangeRate<'a> {
    pub sol: &'a str,
    pub usd: &'a str,
    pub rate: &'a str,
}

#[derive(Template)]
#[template(path = "user-assets.html")]
pub struct AccountAssets {
    pub assets_rows: Vec<application::AssetsRow>,
}
