use application::AssetsRow;
use askama::Template;

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
    pub assets_rows: Vec<AssetsRow>,
}
