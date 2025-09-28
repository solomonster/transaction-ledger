use crate::domain::currency::Currency;

pub type Kobo = i64;

#[derive(Debug, Clone,serde::Serialize,serde::Deserialize)]
pub struct Account {
    pub id: u32,
    pub owner: String,
    pub balance: Kobo,
    pub closed: bool,
    pub currency: Currency,

    #[serde(rename = "bankName", default)]
    pub bank_name: String,

    #[serde(rename = "bankCode", default)]
    pub bank_code: String,

    #[serde(rename = "accountNumber", default)]
    pub account_number: String,
}