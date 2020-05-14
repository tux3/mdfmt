use std::fs::{File, read_to_string};
use std::error::Error;
use std::io::{Write, Read};
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
            .index(1))
        .arg(Arg::with_name("destination")
            .help("Ouptut file (if not inplace)")
            .index(2))
        .get_matches();

    let strict = args.is_present("strict");
    let inplace = args.is_present("inplace");
    if inplace && args.is_present("destination") {
        eprintln!("Cannot be both inplace and have a destination.");
        exit(1);
    }

    let filepath = match args.value_of_os("source") {
        Some(source) if source == "-" => None,
        source => source
    };

    let input_content = if let Some(filepath) = filepath {
        read_to_string(filepath)?
    } else if inplace {
        eprintln!("Cannot be inplace while reading from stdin");
        exit(1);
    } else {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        input
    };

    let formatted = format::format_content(&input_content, strict)?;

    if inplace {
        let mut out_file = File::create(filepath.unwrap())?;
        out_file.write_all(formatted.as_bytes())?;
    } else if let Some(destination) = args.value_of_os("destination") {
        let mut out_file = File::create(destination)?;
        out_file.write_all(formatted.as_bytes())?;
    } else {
        println!("{}", formatted);
    }

    Ok(())
}
