extern crate clap;

use clap::{App, Arg};
use rstdms::TdmsFile;
use std::fs::File;

fn main() {
    match main_impl() {
        Ok(()) => {}
        Err(message) => {
            eprintln!("{}", message);
            std::process::exit(1);
        }
    }
}

fn main_impl() -> Result<(), String> {
    let matches = App::new("tdmsinfo")
        .version("0.0.1")
        .about("Displays TDMS file metadata")
        .arg(
            Arg::with_name("path")
                .help("Path to the TDMS file to read")
                .required(true)
                .index(1),
        )
        .get_matches();

    let path = matches.value_of("path").unwrap();
    let file = match File::open(path) {
        Ok(file) => file,
        Err(err) => {
            return Err(format!("Error opening path {}: {}", path, err));
        }
    };
    let tdms_file = match TdmsFile::new(file) {
        Ok(tdms_file) => tdms_file,
        Err(err) => {
            return Err(format!("Error reading TDMS file {}: {}", path, err));
        }
    };

    for group in tdms_file.groups() {
        println!("{}", group.name());
        for channel in group.channels() {
            println!("{} / {}", group.name(), channel.name());
        }
    }

    Ok(())
}
