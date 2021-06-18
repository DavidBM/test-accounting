use crate::account::{Account, AccountOperation, AccountOperationType};
use crossbeam_channel::{bounded, Sender};
use eyre::Result;
use std::collections::BTreeMap;

/// Each transaction is represented with a AccountCommand::Operate.
/// The Report variant is used to gather the results
/// at the end of the program.
#[derive(Debug)]
pub enum AccountCommand {
    Report(Sender<BTreeMap<u16, Account>>),
    Operate(AccountOperation),
}

/// Handles all the accounts transactions and keeps
/// the state
pub struct AccountProcessor {
    sender: Sender<AccountCommand>,
}

impl AccountProcessor {
    #[inline]
    pub fn new(max_queue: usize) -> AccountProcessor {
        let sender = create_worker(max_queue);

        AccountProcessor { sender }
    }

    /// Executed a transaction. It sends the transaction
    /// operation command to the processing thread via a
    /// channel.
    #[inline]
    pub fn operate(&self, operation: AccountOperation) -> Result<()> {
        self.sender.send(AccountCommand::Operate(operation))?;

        Ok(())
    }

    /// Ends the execution and returns the current state
    /// of all accounts.
    #[inline]
    pub fn report(&self) -> Result<BTreeMap<u16, Account>> {
        let (report_sender, report_receiver) = bounded::<BTreeMap<u16, Account>>(0);

        self.sender.send(AccountCommand::Report(report_sender))?;

        Ok(report_receiver.recv()?)
    }
}

/// Creates a thread to process all command. Including
/// transactions operations and reports
#[inline]
fn create_worker(max_queue: usize) -> Sender<AccountCommand> {
    let (sender, receiver) = bounded::<AccountCommand>(max_queue);

    std::thread::spawn(move || {
        let mut accounts: BTreeMap<u16, Account> = BTreeMap::new();

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

/// Applies the operation to the accounts. If there isn't
/// an account with the provided ID, it will create a new
/// one.
#[inline]
fn apply_operate_command(accounts: &mut BTreeMap<u16, Account>, operation: AccountOperation) {
    if let Some(account) = accounts.get_mut(operation.get_client()) {
        process_operation(account, operation);
    } else {
        let mut account = Account::new(*operation.get_client());
        let client = *operation.get_client();
        process_operation(&mut account, operation);
        accounts.insert(client, account);
    }
}

/// Given an account and an operation, applies the operation
/// to the account
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
