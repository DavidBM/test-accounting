use decimal_rs::Decimal;
use serde::{
    ser::{SerializeStruct, Serializer},
    Serialize,
};
use std::collections::HashMap;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountOperationType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, serde::Deserialize)]
pub struct AccountOperation {
    r#type: AccountOperationType,
    pub client: u16,
    pub tx: u32,
    pub amount: Decimal,
}

impl AccountOperation {
    pub fn get_type(&self) -> &AccountOperationType {
        &self.r#type
    }
}

#[derive(Debug)]
pub struct Account {
    client: u16,
    locked: bool,
    available: Decimal,
    held: Decimal,
    active_disputes: HashMap<u32, Decimal>,
}

impl Serialize for Account {
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
    pub fn new(client: u16) -> Account {
        Account {
            client,
            locked: false,
            available: Decimal::from(0),
            held: Decimal::from(0),
            active_disputes: HashMap::new(),
        }
    }

    pub fn get_total(&self) -> Decimal {
        self.available + self.held
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn lock(&mut self) {
        self.locked = true;
    }

    pub fn deposit(&mut self, amount: Decimal) {
        if self.is_locked() {
            return;
        }

        self.available += amount;
    }

    pub fn withdraw(&mut self, amount: Decimal) {
        if self.is_locked() {
            return;
        }

        self.available -= amount;
    }

    pub fn dispute(&mut self, amount: Decimal, tx_id: u32) {
        if self.is_locked() {
            return;
        }

        self.available -= amount;
        self.held += amount;

        self.active_disputes.insert(tx_id, amount);
    }

    pub fn resolve(&mut self, tx_id: u32) {
        if self.is_locked() {
            return;
        }

        let amount = self.active_disputes.remove(&tx_id);

        if let Some(amount) = amount {
            self.available += amount;
            self.held -= amount;
        }
    }

    pub fn chargeback(&mut self, tx_id: u32) {
        if self.is_locked() {
            return;
        }

        let amount = self.active_disputes.get(&tx_id);

        if amount.is_none() {
            return;
        }

        self.lock();
    }
}
