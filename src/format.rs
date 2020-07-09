use std::error::Error;
use unicode_width::UnicodeWidthStr;

#[derive(Clone)]
enum TableAlignment {
    None,
    Left,
    Center,
    Right,
}

#[derive(Clone)]
struct TableColumn {
    alignment: TableAlignment,
    lines: Vec<String>,
}

struct Table {
    columns: Vec<TableColumn>,
}

enum ParseState {
    RegularText,
    CheckingHeader {
        source_header: String,
        headers: Vec<String>,
    },
    ReadingTable {
        source_table: Vec<String>,
        table: Table,
    },
}

impl ParseState {
    pub fn new() -> Self {
        ParseState::RegularText
    }
}

impl Table {
    pub fn write_output(&self, output: &mut String) {
        let column_widths = self.columns.iter().map(|column| {
           column.lines.iter().map(|l| l.width()).max().unwrap().max(1)
        }).collect::<Vec<_>>();

        let lines = self.columns[0].lines.len();
        self.write_output_line(output, &column_widths, 0);
        self.write_subhead_line(output, &column_widths);
        for i in 1..lines {
            self.write_output_line(output, &column_widths, i);
        }
    }

    fn write_output_line(&self, output: &mut String, widths: &[usize], index: usize) {
        output.push('|');
        for (column, &width) in self.columns.iter().zip(widths) {
            let elem = &column.lines[index];
            let padded = pad_cell_content(elem, width);
            output.push_str(&padded);
            output.push('|');
        }
        output.push('\n');
    }

    fn write_subhead_line(&self, output: &mut String, widths: &[usize]) {
        output.push('|');
        for (column, &width) in self.columns.iter().zip(widths) {
            match column.alignment {
                TableAlignment::Left | TableAlignment::Center => output.push(':'),
                _ => output.push('-'),
            };
            output.push_str(&"-".repeat(width));
            match column.alignment {
                TableAlignment::Right | TableAlignment::Center => output.push(':'),
                _ => output.push('-'),
            };
            output.push('|');
        }
        output.push('\n');
    }
}

pub fn format_content(content: &str, strict: bool) -> Result<String, Box<dyn Error>> {
    let mut result = String::new();

    let mut state = ParseState::new();
    let mut is_in_code = true; // When inside ``` code blocks
    for chunk in content.split("```") {
        is_in_code = !is_in_code;
        if is_in_code {
            result.push_str(&format!("```{}```", chunk));
            continue
        }

        result.push_str(&format_chunk(chunk, &mut state, strict)?);
    }

    if let ParseState::CheckingHeader{source_header, ..} = state {
        result.push_str(&format!("{}\n", source_header));
    } else if let ParseState::ReadingTable{table, ..} = state {
        table.write_output(&mut result);
    }

    Ok(result)
}

/// Returns the formatted chunk (may delay output if a table spans multiple chunks)
fn format_chunk(chunk: &str, state: &mut ParseState, strict: bool) -> Result<String, Box<dyn Error>> {
    let mut output = String::new();

    for line in chunk.lines() {
        *state = match state {
            ParseState::RegularText => process_regular_text(line)?,
            ParseState::CheckingHeader{source_header, headers} => process_header(line, &mut output, source_header, headers)?,
            ParseState::ReadingTable{source_table, table} => process_table(line, &mut output, source_table, table, strict)?,
        };

        if let ParseState::RegularText = state {
            output.push_str(line);
            output.push('\n');
        }
    }

    Ok(output)
}

fn process_regular_text(line: &str) -> Result<ParseState, Box<dyn Error>> {
    let clean = line.trim();
    if !clean.starts_with('|') || !clean.ends_with('|') {
        return Ok(ParseState::RegularText);
    }

    let headers = clean[1..].split_terminator('|').map(|header| header.trim().to_string()).collect::<Vec<_>>();
    if headers.is_empty() {
        return Ok(ParseState::RegularText);
    }

    Ok(ParseState::CheckingHeader {
        source_header: line.to_string(),
        headers,
    })
}

fn process_header(line: &str, output: &mut String, source_header: &str, headers: &[String]) -> Result<ParseState, Box<dyn Error>> {
    let clean = line.trim();
    if !clean.starts_with('|') || !clean.ends_with('|') {
        output.push_str(&format!("{}\n", source_header));
        return Ok(ParseState::RegularText);
    }

    let sub_headers = clean[1..].split_terminator('|').map(|header| header.trim().to_string()).collect::<Vec<_>>();
    if sub_headers.len() != headers.len() {
        output.push_str(&format!("{}\n", source_header));
        return Ok(ParseState::RegularText);
    }

    let mut columns = Vec::new();
    for (header, raw_sub) in headers.iter().zip(sub_headers) {
        let sub = raw_sub.trim();
        if sub.len() < 3 {
            output.push_str(&format!("{}\n", source_header));
            return Ok(ParseState::RegularText);
        }
        let align_left = sub.starts_with(':');
        let align_right = sub.ends_with(':');

        let mut dashes = sub;
        if align_left {
            dashes = &dashes[1..];
        }
        if align_left {
            dashes = &dashes[..dashes.len()-1];
        }
        for c in dashes.chars() {
            if c != '-' {
                output.push_str(&format!("{}\n", source_header));
                return Ok(ParseState::RegularText);
            }
        }

        let alignment = match (align_left, align_right) {
            (false, false) => TableAlignment::None,
            (true, false) => TableAlignment::Left,
            (false, true) => TableAlignment::Right,
            (true, true) => TableAlignment::Center,
        };
        columns.push(TableColumn {
            alignment,
            lines: vec![header.to_owned()],
        })
    }

    Ok(ParseState::ReadingTable {
        source_table: vec![source_header.to_string(), line.to_string()],
        table: Table {
            columns,
        }
    })
}

fn process_table(line: &str, output: &mut String, source_table: &[String], table: &Table, strict: bool) -> Result<ParseState, Box<dyn Error>> {
    let clean = line.trim();
    if !clean.starts_with('|') || !clean.ends_with('|') {
        table.write_output(output);
        return Ok(ParseState::RegularText);
    }

    let columns = clean[1..].split_terminator('|').map(|header| header.trim().to_string()).collect::<Vec<_>>();
    if columns.len() != table.columns.len() {
        // We consider that this is a broken table, not the end of a valid table, so we output the original text
        if strict {
            let line_num = 1 + output.as_bytes().iter().filter(|&&c| c==b'\n').count();
            eprintln!("The table at line {} appears broken, it will not be formatted\n", line_num);
        }
        for line in source_table {
            output.push_str(&format!("{}\n", line));
        }
        return Ok(ParseState::RegularText);
    }

    let mut source_table = source_table.to_vec();
    source_table.push(line.to_owned());

    let mut table_columns = table.columns.clone();
    for (table_column, column) in table_columns.iter_mut().zip(columns) {
        table_column.lines.push(column);
    }

    Ok(ParseState::ReadingTable {
        source_table,
        table: Table {
            columns: table_columns,
        }
    })
}

fn pad_cell_content(elem: &str, width: usize) -> String {
    let mut padded = format!(" {}", elem.trim());
    padded.push_str(&" ".repeat(width + 2 - padded.width()));
    padded
}
