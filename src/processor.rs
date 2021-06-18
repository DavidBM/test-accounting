use std::sync::Arc;
use dashmap::DashMap;
use crate::account::{Account, AccountOperation, AccountOperationType};
use crossbeam_channel::{bounded, Sender};
use eyre::Result;
use rayon::prelude::*;

#[derive(Debug)]
pub enum AccountCommand {
    Report(Sender<Arc<DashMap<u16, Account>>>),
    Operate(AccountOperation),
}

pub struct AccountProcessor {
    sender: Sender<AccountCommand>,
}

impl AccountProcessor {
    #[inline]
    pub fn new(max_queue: usize) -> AccountProcessor {
        let sender = create_worker(max_queue);

        AccountProcessor { sender }
    }

    #[inline]
    pub fn operate(&self, operation: AccountOperation) -> Result<()> {
        self.sender.send(AccountCommand::Operate(operation))?;

        Ok(())
    }

    #[inline]
    pub fn report(&self) -> Result<Arc<DashMap<u16, Account>>> {
        let (report_sender, report_receiver) = bounded(0);

        self.sender.send(AccountCommand::Report(report_sender))?;

        Ok(report_receiver.recv()?)
    }
}

#[inline]
fn create_worker(max_queue: usize) -> Sender<AccountCommand> {
    let (sender, receiver) = bounded::<AccountCommand>(max_queue);

    std::thread::spawn(move || {
        let accounts: Arc<DashMap<u16, Account>> = Arc::new(DashMap::new());

        receiver.into_iter().par_bridge().for_each(|command| {
            match command {
                AccountCommand::Operate(operation) => {
                    apply_operate_command(&accounts, operation)
                }
                AccountCommand::Report(sender) => {
                    let _ = sender.send(accounts.clone());
                }
            }
        });
    });

    sender
}

#[inline]
fn apply_operate_command(accounts: &DashMap<u16, Account>, operation: AccountOperation) {
    if let Some(mut account) = accounts.get_mut(operation.get_client()) {
        process_operation(&mut account, operation);
    } else {
        let mut account = Account::new(*operation.get_client());
        let client = *operation.get_client();
        process_operation(&mut account, operation);
        accounts.insert(client, account);
    }
}

#[inline]
fn process_operation(account: &mut Account, operation: AccountOperation) {
    match operation.get_type() {
        AccountOperationType::Deposit => account.deposit(operation.get_amount()),
        AccountOperationType::Withdrawal => account.withdraw(operation.get_amount()),
        AccountOperationType::Dispute => {
            account.dispute(operation.get_amount(), operation.get_tx())
        }
        AccountOperationType::Resolve => account.resolve(operation.get_tx()),
        AccountOperationType::Chargeback => account.chargeback(operation.get_tx()),
    }
}
