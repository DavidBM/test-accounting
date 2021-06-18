mod account;
mod processor;

use crate::account::AccountOperation;
use crate::processor::AccountProcessor;
use clap::{App, Arg};
use eyre::Result;
use std::io::BufReader;
use std::{convert::TryInto, fs::File};

fn main() -> Result<()> {

    let mut csv_reader = get_source_reader_from_args()?;

    let processor = AccountProcessor::new(1, 10000)?;

    for result in csv_reader.deserialize() {
        let operation: AccountOperation = result?;
        processor.process_operation(operation)?;
    }

    let result = processor.report()?;

    let mut csv_writer = csv::Writer::from_writer(std::io::stdout());

    let _ = result.into_iter().map(|(_, account)| {
        csv_writer.serialize(account).unwrap();
    }).collect::<Vec<()>>();

    Ok(())
}

fn get_source_reader_from_args() -> Result<csv::Reader<BufReader<File>>> {
    let matches = App::new("CSV Transactions Processor")
        .version("1.0")
        .author("David B. <dbmontes@gmail.com>")
        .arg(
            Arg::with_name("source")
                .help("Relative path to the source csv file")
                .required(true)
                .index(1),
        )
        .get_matches();

    let source_csv_path = matches
        .value_of("source")
        .expect("No source csv path provided");

    let source_csv = std::fs::File::open(source_csv_path)?;
    let source_csv = BufReader::new(source_csv);

    Ok(csv::Reader::from_reader(source_csv))
}
