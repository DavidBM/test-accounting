use crate::account::{Account, AccountOperation, AccountOperationType};
use crossbeam_channel::{bounded, Receiver, Sender};
use eyre::Result;
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug)]
pub enum AccountCommand {
    Report(Sender<HashMap<u16, Account>>), //TODO: Add sender here for the final report
    Operate(AccountOperation),
}

pub struct AccountProcessor {
    pool_senders: Vec<Sender<AccountCommand>>,
    pool_size: u16,
}

impl AccountProcessor {
    pub fn new(pool_size: u16, max_queue: usize) -> Result<AccountProcessor> {
        let mut pool_senders: Vec<Sender<AccountCommand>> =
            Vec::with_capacity(usize::try_from(pool_size)?);

        create_threads(&mut pool_senders, max_queue, pool_size);

        Ok(AccountProcessor {
            pool_senders,
            pool_size,
        })
    }

    pub fn process_operation(&self, operation: AccountOperation) -> Result<()> {
        let client_id = operation.client;

        let sender_index = client_id.checked_rem(self.pool_size).unwrap_or(0);

        let sender = self
            .pool_senders
            .get::<usize>(sender_index.into())
            .expect("Cannot find thread to process operation. This is a bug");

        sender.send(AccountCommand::Operate(operation))?;

        Ok(())
    }

    pub fn report(&self) -> Result<HashMap<u16, Account>> {
        let mut receivers: Vec<Receiver<HashMap<u16, Account>>> = vec![];

        for sender in &self.pool_senders {
            let (report_sender, report_receiver) = bounded::<HashMap<u16, Account>>(0);

            sender.send(AccountCommand::Report(report_sender))?;

            receivers.push(report_receiver);
        }

        let mut result: HashMap<u16, Account> = HashMap::new();

        for receiver in receivers {
            result.extend(receiver.recv()?);
        }

        Ok(result)
    }
}

fn create_threads(
    mailboxes: &mut Vec<Sender<AccountCommand>>,
    max_queue: usize,
    pool_size: u16,
) {
    for _ in 0..pool_size {
        let (sender, receiver) = bounded::<AccountCommand>(max_queue);
        mailboxes.push(sender);

        std::thread::spawn(move || {
            let mut accounts: HashMap<u16, Account> = HashMap::new();

            while let Ok(command) = receiver.recv() {
                match command {
                    AccountCommand::Operate(operation) => {
                        ensure_account(&mut accounts, &operation.client);
                        let account = accounts
                            .get_mut(&operation.client)
                            .expect("Cannot find account that was just created. This is a bug");
                        apply_operation(account, operation);
                    }
                    AccountCommand::Report(sender) => {
                        let _ = sender.send(accounts);
                        break;
                    }
                }
            }
        });
    }
}

fn ensure_account(accounts: &mut HashMap<u16, Account>, account_id: &u16) {
    if accounts.get(account_id).is_none() {
        let account = Account::new(*account_id);

        accounts.insert(*account_id, account);
    }
}

fn apply_operation(account: &mut Account, operation: AccountOperation) {
    match operation.get_type() {
        AccountOperationType::Deposit => account.deposit(operation.amount),
        AccountOperationType::Withdrawal => account.withdraw(operation.amount),
        AccountOperationType::Dispute => account.dispute(operation.amount, operation.tx),
        AccountOperationType::Resolve => account.resolve(operation.tx),
        AccountOperationType::Chargeback => account.chargeback(operation.tx)
    }
}
