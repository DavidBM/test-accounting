mod account;
mod processor;

use crate::account::AccountOperation;
use crate::processor::AccountProcessor;
use eyre::Result;
use std::io::{Read, Write};

const QUEUE_CACHE_MB: usize = 25;

/// Processes the transafer in a CSV Reader and outputs
/// the accounts state into a Writer as CSV.
/// 
/// This method is generic over a reader and a writer in
/// order to allow easier testing. It is a nice thing to
/// have too as you can use the code in other libraries
/// and executables too.
pub fn process_csv<R: Read, W: Write>(reader: R, writer: &mut W) -> Result<()> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(reader);

    // This uses a different thread for the operations
    // in order to allow the reader to be as sequential
    // as possible. In the case of the CSV, this allows
    // to have a thread decoding and other processing
    // transactions. The max_queue is a cache of how
    // many messages should be stores before stopping
    // the reader. In my tests I found that 25MB is the
    // ideal, as more can harm the overall time just
    // because the allocation time for the crossbeam-channel
    // initialization.
    let processor = AccountProcessor::new(calculate_mem_cache(QUEUE_CACHE_MB));

    for result in csv_reader.deserialize() {
        let operation: AccountOperation = result?;
        // Each operate call sends a message through a
        // crossbeam channel, so this loop is very thigh
        // and allows the thread to be decoding as much
        // time as possible.
        processor.operate(operation)?;
    }

    // Report sends back the other thread account storage
    // which is in a BTreeMap<u16, Account> (the one that
    // performed better in my testings).
    let result = processor.report()?;

    let mut csv_writer = csv::Writer::from_writer(writer);

    let _ = result
        .into_iter()
        // the main.rs provided a BufWriter with a huge buffer
        // size, so many calls to the serialize method won't
        // hurt the write performance
        .for_each(|(_, account)| csv_writer.serialize(account).unwrap());

    Ok(())
}

/// Calculates how many messages will take to fill the given
/// memory space taking in account the message size sent in
/// the crossbeam channel.
pub fn calculate_mem_cache(megabytes: usize) -> usize {
    let message_size = std::mem::size_of::<AccountOperation>();

    get_mb_in_bytes(megabytes) / message_size
}

pub fn get_mb_in_bytes(mb: usize) -> usize {
    mb * 1024 * 1024
}
