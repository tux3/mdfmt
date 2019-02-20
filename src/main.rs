use std::fs;
use std::error::Error;
use std::process::exit;

mod format;

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Usage: {} <file.md>", args[0]);
        exit(1);
    }

    let filepath = &args[1];
    let content = fs::read_to_string(filepath)?;

    let formatted = format::format_content(&content)?;
    println!("{}", formatted);

    Ok(())
}
