mod account;
mod processor;

use crate::account::AccountOperation;
use crate::processor::AccountProcessor;
use clap::{App, Arg, ArgMatches};
use eyre::Result;
use std::fs::File;
use std::io::{BufReader, BufWriter};

fn main() -> Result<()> {
    let mut csv_reader = get_source_reader_from_args()?;

    let processor = AccountProcessor::new(calculate_mem_cache(1024));

    for result in csv_reader.deserialize() {
        let operation: AccountOperation = result?;
        processor.operate(operation)?;
    }

    let result = processor.report()?;

    let mut csv_writer = csv::Writer::from_writer(BufWriter::with_capacity(
        get_mb_in_bytes(1024),
        std::io::stdout(),
    ));

    let _ = result
        .iter()
        .for_each(|account| csv_writer.serialize(account.value()).unwrap());

    Ok(())
}

fn get_source_reader_from_args() -> Result<csv::Reader<BufReader<File>>> {
    let matches = get_exec_args();

    let source_csv_path = matches
        .value_of("source")
        .expect("No source csv path provided");

    let source_csv = std::fs::File::open(source_csv_path)?;
    let source_csv = BufReader::new(source_csv);

    Ok(csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(source_csv))
}

fn get_exec_args() -> ArgMatches<'static> {
    App::new("CSV Transactions Processor")
        .version("1.0")
        .author("David B. <dbmontes@gmail.com>")
        .arg(
            Arg::with_name("source")
                .help("Relative path to the source csv file")
                .required(true)
                .index(1),
        )
        .get_matches()
}

// Calculates how many messages will take to fill the given
// memory space.
fn calculate_mem_cache(megabytes: usize) -> usize {
    let message_size = std::mem::size_of::<AccountOperation>();

    get_mb_in_bytes(megabytes) / message_size
}

fn get_mb_in_bytes(mb: usize) -> usize {
    mb * 1024 * 1024
}
