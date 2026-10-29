use crate::error::AlsParseError;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::io::BufRead;

/// Resolve the body of an XML reference — the text between `&` and `;`.
///
/// Supports the five predefined entities plus numeric character references
/// (`&#13;` and `&#xD;`). Rave uses the latter to embed carriage returns and
/// line feeds inside cell values, so rejecting them would abort the parse of
/// an otherwise valid workbook.
pub fn resolve_reference(reference: &str) -> Result<char, AlsParseError> {
    resolve_reference_opt(reference).ok_or_else(|| {
        AlsParseError::XmlError(format!("Unsupported XML entity reference: &{reference};"))
    })
}

fn resolve_reference_opt(reference: &str) -> Option<char> {
    match reference {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => numeric_reference(reference),
    }
}

/// Resolve a numeric character reference such as `13` or `xD`.
fn numeric_reference(reference: &str) -> Option<char> {
    let digits = reference.strip_prefix('#')?;
    let code = match digits
        .strip_prefix('x')
        .or_else(|| digits.strip_prefix('X'))
    {
        Some(hex) => u32::from_str_radix(hex, 16).ok()?,
        None => digits.parse::<u32>().ok()?,
    };
    char::from_u32(code)
}

/// Iterator over the rows of an Excel SSXML worksheet.
///
/// Every row is returned as the list of its cell values, indexed by their
/// 0-based column. A cell carrying an `ss:Index` attribute is placed at that
/// column — the attribute is 1-based, so `ss:Index="6"` is column 5 — while
/// every other cell follows the one before it. Columns that hold no cell stay
/// empty.
///
/// Cell values are collected verbatim: the text of a `<Data>` element is
/// assembled from the text events *and* the entity references quick-xml emits
/// separately, so a value such as `a &gt; b` is kept intact instead of being
/// cut into pieces.
pub struct SheetRows<'r, R: BufRead> {
    reader: &'r mut Reader<R>,
    buffer: Vec<u8>,
    /// Values of the row currently being assembled.
    row: Vec<String>,
    /// Text collected for the cell currently being read.
    cell: String,
    /// 0-based column of the cell currently being read.
    column: usize,
    /// 0-based column the next cell without an `ss:Index` will occupy.
    next_column: usize,
    /// Whether the reader is inside a `<Data>` element.
    in_data: bool,
    finished: bool,
}

impl<'r, R: BufRead> SheetRows<'r, R> {
    pub fn new(reader: &'r mut Reader<R>) -> Self {
        Self {
            reader,
            buffer: Vec::new(),
            row: Vec::new(),
            cell: String::new(),
            column: 0,
            next_column: 0,
            in_data: false,
            finished: false,
        }
    }

    /// Store the cell that was just read and move on to the next column.
    fn close_cell(&mut self) {
        while self.row.len() <= self.column {
            self.row.push(String::new());
        }
        self.row[self.column] = std::mem::take(&mut self.cell);
        self.next_column = self.column + 1;
    }
}

impl<'r, R: BufRead> Iterator for SheetRows<'r, R> {
    type Item = Result<Vec<String>, AlsParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        loop {
            self.buffer.clear();
            let event = match self.reader.read_event_into(&mut self.buffer) {
                Ok(event) => event,
                Err(e) => {
                    self.finished = true;
                    return Some(Err(AlsParseError::XmlError(e.to_string())));
                }
            };

            match event {
                Event::Eof => {
                    self.finished = true;
                    return None;
                }
                Event::Start(e) => match e.name().as_ref() {
                    b"Row" => {
                        self.row.clear();
                        self.next_column = 0;
                    }
                    b"Cell" => {
                        self.column = cell_column(&e).unwrap_or(self.next_column);
                        self.cell.clear();
                    }
                    b"Data" => self.in_data = true,
                    _ => {}
                },
                // An empty `<Cell/>` still occupies a column.
                Event::Empty(e) if e.name().as_ref() == b"Cell" => {
                    self.column = cell_column(&e).unwrap_or(self.next_column);
                    self.cell.clear();
                    self.close_cell();
                }
                Event::End(e) => match e.name().as_ref() {
                    b"Data" => self.in_data = false,
                    b"Cell" => self.close_cell(),
                    b"Row" => return Some(Ok(std::mem::take(&mut self.row))),
                    b"Worksheet" => {
                        self.finished = true;
                        return None;
                    }
                    _ => {}
                },
                Event::Text(e) => {
                    if self.in_data {
                        let decoded = match e.decode() {
                            Ok(decoded) => decoded,
                            Err(e) => {
                                self.finished = true;
                                return Some(Err(AlsParseError::XmlError(e.to_string())));
                            }
                        };
                        self.cell.push_str(&decoded);
                    }
                }
                Event::GeneralRef(e) => {
                    if !self.in_data {
                        continue;
                    }
                    let reference = match std::str::from_utf8(e.as_ref()) {
                        Ok(reference) => reference,
                        Err(e) => {
                            self.finished = true;
                            return Some(Err(AlsParseError::XmlError(e.to_string())));
                        }
                    };
                    match resolve_reference(reference) {
                        Ok(character) => self.cell.push(character),
                        Err(e) => {
                            self.finished = true;
                            return Some(Err(e));
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// Read the 0-based column a cell is placed in from its `ss:Index` attribute.
fn cell_column(start: &BytesStart) -> Option<usize> {
    for attr in start.attributes().flatten() {
        if attr.key.as_ref() == b"ss:Index" || attr.key.as_ref() == b"Index" {
            let value = std::str::from_utf8(attr.value.as_ref()).ok()?;
            if let Ok(index) = value.parse::<usize>() {
                return Some(index.saturating_sub(1));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows_of(xml: &str) -> Result<Vec<Vec<String>>, AlsParseError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().expand_empty_elements = true;
        SheetRows::new(&mut reader).collect()
    }

    #[test]
    fn predefined_entities_are_resolved() {
        assert_eq!(resolve_reference("amp").unwrap(), '&');
        assert_eq!(resolve_reference("lt").unwrap(), '<');
        assert_eq!(resolve_reference("gt").unwrap(), '>');
        assert_eq!(resolve_reference("quot").unwrap(), '"');
        assert_eq!(resolve_reference("apos").unwrap(), '\'');
    }

    #[test]
    fn numeric_character_references_are_resolved() {
        assert_eq!(resolve_reference("#13").unwrap(), '\r');
        assert_eq!(resolve_reference("#10").unwrap(), '\n');
        assert_eq!(resolve_reference("#9").unwrap(), '\t');
        assert_eq!(resolve_reference("#xD").unwrap(), '\r');
        assert_eq!(resolve_reference("#XA").unwrap(), '\n');
        assert_eq!(resolve_reference("#128512").unwrap(), '\u{1F600}');
    }

    #[test]
    fn unknown_entities_are_rejected() {
        assert!(resolve_reference("nbsp").is_err());
        assert!(resolve_reference("#").is_err());
        assert!(resolve_reference("#xZZ").is_err());
        // Surrogates are not valid characters.
        assert!(resolve_reference("#xD800").is_err());
    }

    #[test]
    fn rows_place_cells_by_ss_index() {
        let xml = r#"<Worksheet ss:Name="Forms">
            <Row>
                <Cell><Data>A</Data></Cell>
                <Cell ss:Index="4"><Data>D</Data></Cell>
                <Cell><Data>E</Data></Cell>
            </Row>
        </Worksheet>"#;

        let rows = rows_of(xml).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0], vec!["A", "", "", "D", "E"]);
    }

    #[test]
    fn rows_keep_entity_references_intact() {
        let xml = r#"<Worksheet ss:Name="Forms">
            <Row>
                <Cell><Data>grade&gt;= 3</Data></Cell>
                <Cell><Data>line1&#13;&#10;line2</Data></Cell>
            </Row>
        </Worksheet>"#;

        let rows = rows_of(xml).unwrap();
        assert_eq!(rows[0][0], "grade>= 3");
        assert_eq!(rows[0][1], "line1\r\nline2");
    }

    #[test]
    fn rows_stop_at_the_end_of_the_worksheet() {
        let xml = r#"<Worksheet ss:Name="Forms">
            <Row><Cell><Data>A</Data></Cell></Row>
        </Worksheet>
        <Worksheet ss:Name="Other">
            <Row><Cell><Data>B</Data></Cell></Row>
        </Worksheet>"#;

        let rows = rows_of(xml).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], "A");
    }
}
