use reqwest;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub struct Resp {
    pub date_info: String,
    pub prices: HashMap<String, OilPrice>,
}

#[derive(Debug)]
pub struct Price {
    pub value: String,
    pub change: String,
}

impl Price {
    pub fn new_default() -> Self {
        Price {
            value: String::from(""),
            change: String::from(""),
        }
    }

    pub fn new(value: String, change: String) -> Self {
        Price { value, change }
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " 💲{}  🔄{}", self.value, self.change)
    }
}
#[derive(Debug)]
pub struct OilPrice {
    pub regular: Price,
    pub premium: Price,
    pub diesel: Price,
}

impl OilPrice {
    pub fn new_default() -> Self {
        OilPrice {
            regular: Price::new_default(),
            premium: Price::new_default(),
            diesel: Price::new_default(),
        }
    }

    pub fn new(regular: Price, premium: Price, diesel: Price) -> Self {
        OilPrice {
            regular,
            premium,
            diesel,
        }
    }
    
}

impl fmt::Display for OilPrice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "⛽ **R**:     {}\n\
             🚗 **P**:     {}\n\
             🚚 **D**:     {}",
            self.regular, self.premium, self.diesel
        )
    }
}

pub async fn get_prices() -> Result<Resp, Box<dyn std::error::Error>> {
    let url = "https://gaswizard.ca/gas-price-predictions/";
    let resp = reqwest::get(url).await?;
    let body = resp.text().await?;
    let fragment = Html::parse_document(&body);

    let city_price_rows = Selector::parse("tr.city").unwrap();
    let city_element_selector = Selector::parse("td.gwgp-cityname").unwrap();
    let price_element_selector = Selector::parse("td.gwgp-price").unwrap();
    let title_selector = Selector::parse("div.price-date").unwrap();

    let title_name = fragment
        .select(&title_selector)
        .next()
        .map_or(None, |e| Some(e.inner_html()))
        .unwrap_or_else(|| String::from("N/A"));

    let mut prices = HashMap::new();
    for row in fragment.select(&city_price_rows) {
        let mut city = String::from("");
        let price_reg: Vec<_>;
        let price_pre: Vec<_>;
        let price_dis: Vec<_>;
        let mut price: OilPrice = OilPrice::new_default();

        if let Some(city_name_raw) = row.select(&city_element_selector).next() {
            city = city_name_raw.inner_html().trim().to_string();
            city.truncate(city.len()-1);
        } else {
            continue;
        }

        let mut price_raw = row.select(&price_element_selector);
        if let Some(price_reg_raw) = price_raw.next() {
            price_reg = price_reg_raw.text().collect();
            price.regular = Price::new(
                price_reg[0].trim().to_string(),
                price_reg[1].trim().to_string(),
            );
        } else {
            continue;
        }

        if let Some(price_pre_raw) = price_raw.next() {
            price_pre = price_pre_raw.text().collect();
            price.premium = Price::new(
                price_pre[0].trim().to_string(),
                price_pre[1].trim().to_string(),
            );
        } else {
            continue;
        }

        if let Some(price_dis_raw) = price_raw.next() {
            price_dis = price_dis_raw.text().collect();
            price.diesel = Price::new(
                price_dis[0].trim().to_string(),
                price_dis[1].trim().to_string(),
            );
        } else {
            continue;
        }

        prices.insert(city, price);
    }

    Ok(Resp{date_info: title_name, prices})
}
