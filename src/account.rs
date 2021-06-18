use decimal_rs::Decimal;
use serde::{
    ser::{SerializeStruct, Serializer},
    Serialize,
};
use std::collections::HashMap;

/// Representes all 5 transaction types
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountOperationType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// Represents a transaction to be applied to an account.
#[derive(Debug, serde::Deserialize)]
pub struct AccountOperation {
    r#type: AccountOperationType,
    client: u16,
    tx: u32,
    amount: Option<Decimal>,
}

impl AccountOperation {
    #[inline]
    pub fn get_type(&self) -> &AccountOperationType {
        &self.r#type
    }

    #[inline]
    pub fn get_amount(&self) -> &Option<Decimal> {
        &self.amount
    }

    #[inline]
    pub fn get_tx(&self) -> &u32 {
        &self.tx
    }

    #[inline]
    pub fn get_client(&self) -> &u16 {
        &self.client
    }
}

/// Represents an account where transactions can be applied.
/// It has a custome serializer so it can be rendered only
/// with the human-expected arguments.
#[derive(Debug)]
pub struct Account {
    client: u16,
    locked: bool,
    available: Decimal,
    held: Decimal,
    active_disputes: HashMap<u32, Decimal>,
}

impl Serialize for Account {
    #[inline]
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut output = ser.serialize_struct("Account", 5)?;
        output.serialize_field("client", &self.client)?;
        output.serialize_field("available", &self.available)?;
        output.serialize_field("held", &self.held)?;
        output.serialize_field("locked", &self.locked)?;
        output.serialize_field("total", &self.get_total())?;

        output.end()
    }
}

impl Account {
    #[inline]
    pub fn new(client: u16) -> Account {
        Account {
            client,
            locked: false,
            available: Decimal::from(0),
            held: Decimal::from(0),
            active_disputes: HashMap::new(),
        }
    }

    #[inline]
    pub fn get_total(&self) -> Decimal {
        self.available + self.held
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        self.locked
    }

    #[inline]
    pub fn lock(&mut self) {
        self.locked = true;
    }

    #[inline]
    pub fn deposit(&mut self, amount: &Option<Decimal>) {
        if self.is_locked() {
            return;
        }

        if let Some(amount) = amount {
            self.available += amount;
        }
    }

    #[inline]
    pub fn withdraw(&mut self, amount: &Option<Decimal>) {
        if self.is_locked() {
            return;
        }

        if let Some(amount) = amount {
            if self.available < amount {
                return;
            }

            self.available -= amount;
        }
    }

    #[inline]
    pub fn dispute(&mut self, amount: &Option<Decimal>, tx_id: &u32) {
        if self.is_locked() {
            return;
        }

        if let Some(amount) = amount {
            self.available -= amount;
            self.held += amount;

            self.active_disputes.insert(*tx_id, *amount);
        }
    }

    #[inline]
    pub fn resolve(&mut self, tx_id: &u32) {
        if self.is_locked() {
            return;
        }

        let amount = self.active_disputes.remove(&tx_id);

        if let Some(amount) = amount {
            self.available += amount;
            self.held -= amount;
        }
    }

    #[inline]
    pub fn chargeback(&mut self, tx_id: &u32) {
        if self.is_locked() {
            return;
        }

        // We keep the dispute for later research of why
        // the account was locked
        let amount = self.active_disputes.get(&tx_id);

        if amount.is_none() {
            return;
        }

        self.lock();
    }
}
