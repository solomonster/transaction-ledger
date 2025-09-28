use crate::domain::{account::Kobo, currency::{self, Currency}};

use super::{account::Account, transaction::{Transaction, TransactionEntry}};
use std::collections::HashMap;
use serde::{Serialize,Deserialize};
use tokio::fs;

/// Small utility to format kobo -> ₦x.yy
fn format_naira(k: Kobo) -> String {
    format!("₦{:.2}", (k as f64) / 100.0)
}

#[derive(Debug, Serialize,Deserialize)]
 pub struct Ledger {
    pub accounts: HashMap<u32,Account>,
    pub transactions: Vec<Transaction>,
    pub next_account_id: u32,
    pub next_tx_id: u64,
    pub bank_account_id:u32,
 }
impl Ledger {
    pub fn new() -> Self {

        let mut accounts = HashMap::new();
        let bank = Account {
            id:0,
            owner: "BANK".to_string(),
            balance:0,
            closed: false,
        };
        accounts.insert(0, bank);
        Ledger { 
            accounts,
            transactions: Vec::new(),
            next_account_id: 1,
            next_tx_id: 1, 
            bank_account_id: 0,
        }
    }

    pub fn create_account(&mut self, owner: String, initial_balance: Kobo, currency: Currency)-> Result<u32,String> {
        let id =  self.next_account_id;
        let account = Account {
            id,
            owner,
            balance: initial_balance,
            closed: false,
            currency,
        };
        self.accounts.insert(id, account);
        self.next_account_id = self
        .next_account_id
        .checked_add(1)
        .ok_or("Account id overflow")?;
        Ok(id)
    }
    pub fn close_account(&mut self, account_id: u32)-> Result<(),String> {

        let acc = self
        .accounts.get_mut(&account_id)
        .ok_or_else(|| format!("Account {} not found",account_id))?;
        if acc.balance != 0 {
            return Err(format!("Cannot close acount {}: balance not zero ({}) ",account_id,format_naira(acc.balance)));
        }
        acc.closed = true;
        Ok(())
    }

    pub fn record_transaction(
        &mut self,
        description: Option<String>,
        entries: Vec<TransactionEntry>,
    ) -> Result<u64,String>{

        if entries.is_empty() {
            return Err("Entry must have at least 1 transaction".into());
        }
        let sum_debits: Kobo = entries.iter().map(|e| e.debit).sum();
        let sum_credits: Kobo = entries.iter().map(|e| e.credit).sum();
        if sum_debits != sum_credits {
            return Err(format!(
                "Unbalanced transactions debit:{} credit:{}",
                sum_debits,sum_credits
            ));
        }
        // Check accounts exist and are open
        for e in &entries {
            if let Some(acc) = self.accounts.get(&e.account_id) {
                if acc.closed {
                    return Err(format!("Account {} is closed", e.account_id));
                }
            } 
            else {
                return Err(format!("Account {} does not exist", e.account_id));
            }
        }

        for e in &entries {
            let acc = self
            .accounts
            .get_mut(&e.account_id)
            .expect("account checked above");

            acc.balance = acc.balance.checked_add(e.debit).and_then(|b| b.checked_sub(e.credit)).ok_or_else(|| "overflow applying entry".to_string())?;
        }

        let tx_id = self.next_tx_id;
        
        let tx = Transaction{
            id: tx_id,
            description,
            entries,
            timestamp: chrono::Utc::now(),
        };
        self.transactions.push(tx);
        self.next_tx_id = self.next_tx_id.checked_add(1).ok_or("Transaction id overflow")?;
        Ok(tx_id)

    }


    pub fn deposit(&mut self, to_id: u32, amount:Kobo,description: Option<String>)->Result<u64,String> {
        let entries = vec![
            TransactionEntry {
                account_id:to_id,
                debit:amount,
                credit:0,
            },
            TransactionEntry {
                account_id: self.bank_account_id,
                debit:0,
                credit:amount,
            }
        ];


        self.record_transaction(description, entries)
    }

    pub fn withdraw(&mut self, from_id: u32, amount:Kobo, description: Option<String>)-> Result<u64,String>
    {
        if amount <= 0 {
            return Err("Withdrawal amount must be positive".into());
        }
        let bal = self.get_balance(from_id).ok_or("Account not found")?;
        if bal < amount {
            return Err("Insufficient funds".into());
        }
        let entries = vec![
            TransactionEntry {
                account_id:from_id,
                debit:0,
                credit:amount,
            },
            TransactionEntry {
                account_id:self.bank_account_id,
                debit:amount,
                credit:0,
            },
        ];
        self.record_transaction(description, entries)
        
    }

    pub fn transfer(&mut self, from_id: u32, to_id: u32, amount:Kobo, description: Option<String>)-> Result<u64, String>
    {
        if amount <= 0 {
            return Err("Transfer amount must be positive".into());
        }

        if from_id == to_id {
            return Err("Cannot transfer to the same account".into());
        }


       let bal = self.get_balance(from_id).ok_or("Source account not found")?;
        if bal < amount {
            return Err("Insufficient funds".into());
        }

        let entries = vec![
            TransactionEntry {
                account_id: to_id,
                debit: amount,
                credit: 0,
            },
            TransactionEntry {
                account_id: from_id,
                debit: 0,
                credit: amount,
            },
        ];
        self.record_transaction(description, entries)

    }

    pub fn get_balance(&self,account_id: u32)-> Option<Kobo> {
        self.accounts.get(&account_id).map(|acc| acc.balance)
    }

    pub fn find_account_by_owner(&self, name:&str)-> Option<&Account> {
        self.accounts.values().find(|acc| acc.owner == name)
    }


    pub fn total_assets(&self) -> Kobo {
        self.accounts.values().filter(|a| a.id != self.bank_account_id).map(|a| a.balance).sum()
        
    }

    pub fn richest_account(&self)-> Option<&Account>{
        self.accounts.values().filter(|a| a.id != self.bank_account_id).max_by_key(|a| a.balance)

    }

    pub fn transactions_for_account(&self, account_id: u32)-> Vec<&Transaction> {

        self.transactions.iter().filter(|tx| tx.entries.iter().any(|e| e.account_id== account_id)).collect()
        
    }
    pub async fn save_to_file(&self, path: &std::path::Path) -> Result<(),String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        tokio::fs::write(path, json).await.map_err(|e| e.to_string())?;
        Ok(())
    }
    pub async fn load_from_file(path: &std::path::Path) -> Result<Self,String> {
        let s = tokio::fs::read_to_string(path).await.map_err(|e| e.to_string())?;
        let ledger: Ledger = serde_json::from_str(&s).map_err(|e| e.to_string())?;
        Ok(ledger)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    
   
}
