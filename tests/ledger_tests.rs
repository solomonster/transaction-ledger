use transaction_ledger::domain::currency::Currency;
use transaction_ledger::domain::ledger::{Ledger};
use transaction_ledger::domain::account::Kobo;

 #[test]
    fn test_deposit_increases_balance() {
        let mut ledger = Ledger::new();
        let currency = Currency::NGN;
        // Create an account for Alice with balance 0
        let alice_id = ledger.create_account("Alice".to_string(), 0,currency).unwrap();

        // Deposit 1000 into Alice's account
        let tx_id = ledger.deposit(alice_id, 1000, Some("First deposit".to_string())).unwrap();

        // Verify transaction ID is 1
        assert_eq!(tx_id, 1);

        // Verify Alice's balance increased
        let balance = ledger.get_balance(alice_id).unwrap();
        assert_eq!(balance, 1000);

        // Verify bank account decreased accordingly
        let bank_balance = ledger.get_balance(ledger.bank_account_id).unwrap();
        assert_eq!(bank_balance, -1000); // bank started 0, gave away 1000, so balance is -1000 in accounting sense
    }

    #[test]
    fn test_withdraw_reduces_balance() {
        let mut ledger = Ledger::new();
        let currency = Currency::NGN;
        let bob_id = ledger.create_account("Bob".to_string(), 2000,currency).unwrap();

        // Withdraw 500
        ledger.withdraw(bob_id, 500, Some("ATM withdrawal".to_string())).unwrap();

        // Bob's balance reduced
        assert_eq!(ledger.get_balance(bob_id).unwrap(), 1500);
    }

    #[test]
    fn test_transfer_between_accounts() {
        let mut ledger = Ledger::new();
        let currency = Currency::NGN;
        let alice_id = ledger.create_account("Alice".to_string(), 1000,currency.clone()).unwrap();
        let bob_id = ledger.create_account("Bob".to_string(), 500,currency).unwrap();

        // Transfer 300 from Alice to Bob
        ledger.transfer(alice_id, bob_id, 300, Some("Payback".to_string())).unwrap();

        assert_eq!(ledger.get_balance(alice_id).unwrap(), 700);
        assert_eq!(ledger.get_balance(bob_id).unwrap(), 800);
    }