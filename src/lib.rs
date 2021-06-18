mod account;
mod processor;

use crate::account::AccountOperation;
use crate::processor::AccountProcessor;
use eyre::Result;
use std::io::{Read, Write};

pub fn process_csv<R: Read, W: Write>(reader: R, writer: &mut W) -> Result<()> {
    let mut csv_reader = csv::ReaderBuilder::new().flexible(true).from_reader(reader);

    // 25MB as more cna harm the overall time just because allocation time
    // for the corssbeam-channel
    let processor = AccountProcessor::new(calculate_mem_cache(25));

    for result in csv_reader.deserialize() {
        let operation: AccountOperation = result?;
        processor.operate(operation)?;
    }

    let result = processor.report()?;

    let mut csv_writer = csv::Writer::from_writer(writer);

    let _ = result
        .into_iter()
        .for_each(|(_, account)| csv_writer.serialize(account).unwrap());

    Ok(())
}

/// Calculates how many messages will take to fill the given
/// memory space.
pub fn calculate_mem_cache(megabytes: usize) -> usize {
    let message_size = std::mem::size_of::<AccountOperation>();

    get_mb_in_bytes(megabytes) / message_size
}

pub fn get_mb_in_bytes(mb: usize) -> usize {
    mb * 1024 * 1024
}
