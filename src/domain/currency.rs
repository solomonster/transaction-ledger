use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Currency {
    NGN, // Nigerian Naira
    USD, // US Dollar
    EUR, // Euro
    GBP, // British Pound
}