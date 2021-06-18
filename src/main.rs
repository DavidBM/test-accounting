use clap::{App, Arg, ArgMatches};
use eyre::Result;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
};
use test_accounting::{get_mb_in_bytes, process_csv};

fn main() -> Result<()> {
    let source_reader = get_source_reader_from_args()?;

    let mut output_writer = BufWriter::with_capacity(get_mb_in_bytes(1024), std::io::stdout());

    process_csv(source_reader, &mut output_writer)?;

    Ok(())
}

/// Return a reader from a file given from the shell
/// first argument
fn get_source_reader_from_args() -> Result<BufReader<File>> {
    let matches = get_exec_args();

    let source_csv_path = matches
        .value_of("source")
        .expect("No source csv path provided");

    let source_csv = std::fs::File::open(source_csv_path)?;

    Ok(BufReader::new(source_csv))
}

/// Declares the binary executable parameters and help
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
