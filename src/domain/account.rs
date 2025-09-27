pub type Kobo = i64;

#[derive(Debug, Clone,serde::Serialize,serde::Deserialize)]
pub struct Account {
    pub id: u32,
    pub owner: String,
    pub balance: Kobo,
    pub closed: bool,
}