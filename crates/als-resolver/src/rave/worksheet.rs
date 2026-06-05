use quick_xml::events::Event;
use quick_xml::Reader;
use crate::error::AlsParseError;

/// Navigates to a specific worksheet in the Excel SSXML format.
pub struct WorksheetNavigator<R: std::io::Read> {
    reader: Reader<R>,
    buffer: Vec<u8>,
}

impl<R: std::io::Read> WorksheetNavigator<R> {
    pub fn new(reader: Reader<R>) -> Self {
        Self {
            reader,
            buffer: Vec::new(),
        }
    }

    /// Navigate to a worksheet by name. Returns position byte offset.
    pub fn find_worksheet(&mut self, name: &str) -> Result<usize, AlsParseError> {
        // Reset to beginning
        self.reader.reset();
        self.buffer.clear();

        let mut bytes_read = 0;
        loop {
            self.buffer.clear();
            match self.reader.read_event_into(&mut self.buffer) {
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) if e.name().as_ref() == b"Worksheet" => {
                    // Check ss:Name attribute
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"ss:Name" || attr.key.as_ref() == b"Name" {
                            if attr.value.as_ref() == name.as_bytes() {
                                return Ok(bytes_read);
                            }
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => return Err(AlsParseError::XmlError(e.to_string())),
            }
            bytes_read += self.buffer.len();
        }

        Err(AlsParseError::WorksheetNotFound(name.to_string()))
    }

    /// Get a reference to the underlying reader
    pub fn reader(&self) -> &Reader<R> {
        &self.reader
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_find_worksheet() {
        let xml = br#"<?xml version="1.0"?>
<Workbook>
  <Worksheet ss:Name="Forms">
    <Table><Row><Cell><Data>SC</Data></Cell></Row></Table>
  </Worksheet>
</Workbook>"#;
        let cursor = Cursor::new(xml);
        let reader = Reader::from_reader(cursor);
        let mut nav = WorksheetNavigator::new(reader);
        let pos = nav.find_worksheet("Forms").unwrap();
        assert!(pos > 0);
    }

    #[test]
    fn test_worksheet_not_found() {
        let xml = br#"<?xml version="1.0"?>
<Workbook>
  <Worksheet ss:Name="Forms">
    <Table><Row><Cell><Data>SC</Data></Cell></Row></Table>
  </Worksheet>
</Workbook>"#;
        let cursor = Cursor::new(xml);
        let reader = Reader::from_reader(cursor);
        let mut nav = WorksheetNavigator::new(reader);
        let result = nav.find_worksheet("NonExistent");
        assert!(matches!(result, Err(AlsParseError::WorksheetNotFound(_))));
    }
}