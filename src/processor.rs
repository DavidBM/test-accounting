use crate::account::{Account, AccountOperation, AccountOperationType};
use std::collections::HashMap;


pub struct AccountProcessor {
    accounts: HashMap<u16, Account>,
}

impl AccountProcessor {
    pub fn new() -> AccountProcessor {
        AccountProcessor {
            accounts: HashMap::with_capacity(u16::MAX.into()),
        }
    }

    pub fn process_operation(&mut self, operation: AccountOperation) {
        if let Some(account) = self.accounts.get_mut(&operation.client) {
            apply_operation(account, &operation);
        } else {
            let mut account = Account::new(operation.client);
            apply_operation(&mut account, &operation);
            self.accounts.insert(operation.client, account);
        }
    }

    pub fn report(&self) -> &HashMap<u16, Account> {
        &self.accounts
    }
}


fn apply_operation(account: &mut Account, operation: &AccountOperation) {
    match operation.get_type() {
        AccountOperationType::Deposit => account.deposit(operation.amount),
        AccountOperationType::Withdrawal => account.withdraw(operation.amount),
        AccountOperationType::Dispute => account.dispute(operation.amount, operation.tx),
        AccountOperationType::Resolve => account.resolve(operation.tx),
        AccountOperationType::Chargeback => account.chargeback(operation.tx)
    }
}
