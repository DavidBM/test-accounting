use crate::account::{Account, AccountOperation, AccountOperationType};
use crossbeam_channel::{bounded, Sender};
use eyre::Result;
use std::collections::HashMap;

#[derive(Debug)]
pub enum AccountCommand {
    Report(Sender<HashMap<u16, Account>>), //TODO: Add sender here for the final report
    Operate(AccountOperation),
}

pub struct AccountProcessor {
    sender: Sender<AccountCommand>,
}

impl AccountProcessor {
    pub fn new(max_queue: usize) -> AccountProcessor {
        let sender = create_thread(max_queue);

        AccountProcessor { sender }
    }

    pub fn operate(&self, operation: AccountOperation) -> Result<()> {
        self.sender.send(AccountCommand::Operate(operation))?;

        Ok(())
    }

    pub fn report(&self) -> Result<HashMap<u16, Account>> {
        let (report_sender, report_receiver) = bounded::<HashMap<u16, Account>>(0);

        self.sender.send(AccountCommand::Report(report_sender))?;

        Ok(report_receiver.recv()?)
    }
}

fn create_thread(max_queue: usize) -> Sender<AccountCommand> {
    let (sender, receiver) = bounded::<AccountCommand>(max_queue);

    std::thread::spawn(move || {
        let mut accounts: HashMap<u16, Account> = HashMap::new();

        while let Ok(command) = receiver.recv() {
            match command {
                AccountCommand::Operate(operation) => {
                    apply_operate_command(&mut accounts, operation)
                }
                AccountCommand::Report(sender) => {
                    let _ = sender.send(accounts);
                    break;
                }
            }
        }
    });

    sender
}

fn apply_operate_command(accounts: &mut HashMap<u16, Account>, operation: AccountOperation) {
    if let Some(account) = accounts.get_mut(&operation.client) {
        process_operation(account, operation);
    } else {
        let mut account = Account::new(operation.client);
        process_operation(&mut account, operation);
    }
}

fn process_operation(account: &mut Account, operation: AccountOperation) {
    match operation.get_type() {
        AccountOperationType::Deposit => account.deposit(operation.amount),
        AccountOperationType::Withdrawal => account.withdraw(operation.amount),
        AccountOperationType::Dispute => account.dispute(operation.amount, operation.tx),
        AccountOperationType::Resolve => account.resolve(operation.tx),
        AccountOperationType::Chargeback => account.chargeback(operation.tx),
    }
}
