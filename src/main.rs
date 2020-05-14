use std::fs::{File, read_to_string};
use std::error::Error;
use std::io::Write;
use std::process::exit;
use clap::{Arg, App};

mod format;

fn main() -> Result<(), Box<dyn Error>> {
    let args = App::new("mdfmt")
        .about("Markdown Formatter")
        .arg(Arg::with_name("inplace")
            .short("i")
            .long("in-place")
            .help("Modify input file in place"))
        .arg(Arg::with_name("strict")
            .short("s")
            .long("strict")
            .help("Warn if an input file contains broken tables (instead of ignoring them)"))
        .arg(Arg::with_name("source")
            .help("The source file to format")
            .required(true)
            .index(1))
        .arg(Arg::with_name("destination")
            .help("Ouptut file (if not inplace)")
            .index(2))
        .get_matches();

    let strict = args.is_present("strict");
    let inplace = args.is_present("inplace");
    if inplace && args.is_present("destination") {
        println!("Cannot be both inplace and have a destination.");
        exit(1);
    }

    let filepath = args.value_of_os("source").unwrap();
    let content = read_to_string(filepath)?;

    let formatted = format::format_content(&content, strict)?;

    if inplace {
        let mut out_file = File::create(filepath)?;
        out_file.write_all(formatted.as_bytes())?;
    } else if let Some(destination) = args.value_of_os("destination") {
        let mut out_file = File::create(destination)?;
        out_file.write_all(formatted.as_bytes())?;
    } else {
        println!("{}", formatted);
    }

    Ok(())
}
