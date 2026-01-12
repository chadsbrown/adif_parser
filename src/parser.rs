use crate::error::{AdifError, Result};
use crate::types::{AdifFile, AdifHeader, DataType, Field, Record};

/// Parse an ADI format string into an AdifFile
pub fn parse_adi(input: &str) -> Result<AdifFile> {
    let mut parser = AdiParser::new(input);
    parser.parse()
}

/// Internal parser state
struct AdiParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> AdiParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse(&mut self) -> Result<AdifFile> {
        let mut file = AdifFile::new();

        // Check if there's a header by looking for <EOH>
        let has_header = self.input.to_uppercase().contains("<EOH>");

        if has_header {
            file.header = self.parse_header()?;
        }

        // Parse records
        file.records = self.parse_records()?;

        Ok(file)
    }

    fn parse_header(&mut self) -> Result<AdifHeader> {
        let mut header = AdifHeader::default();

        // Capture any preamble text before the first tag
        let preamble_end = self.find_next_tag_start().unwrap_or(self.input.len());
        header.preamble = self.input[..preamble_end].to_string();
        self.pos = preamble_end;

        // Parse header fields until we hit <EOH>
        loop {
            self.skip_whitespace_and_newlines();

            if self.pos >= self.input.len() {
                break;
            }

            // Look for the next tag
            if self.peek_char() != Some('<') {
                self.pos += 1;
                continue;
            }

            // Check for EOH
            if self.check_tag("EOH") {
                self.skip_tag("EOH")?;
                break;
            }

            // Parse a field
            let field = self.parse_field()?;

            // Extract well-known header fields
            match field.name.as_str() {
                "ADIF_VER" => header.adif_version = Some(field.value.clone()),
                "PROGRAMID" => header.program_id = Some(field.value.clone()),
                "PROGRAMVERSION" => header.program_version = Some(field.value.clone()),
                "CREATED_TIMESTAMP" => header.created_timestamp = Some(field.value.clone()),
                _ => {}
            }

            header.fields.push(field);
        }

        Ok(header)
    }

    fn parse_records(&mut self) -> Result<Vec<Record>> {
        let mut records = Vec::new();
        let mut current_record = Record::new();

        loop {
            self.skip_whitespace_and_newlines();

            if self.pos >= self.input.len() {
                break;
            }

            // Look for the next tag
            if self.peek_char() != Some('<') {
                self.pos += 1;
                continue;
            }

            // Check for EOR (End of Record)
            if self.check_tag("EOR") {
                self.skip_tag("EOR")?;
                if !current_record.fields.is_empty() {
                    records.push(current_record);
                    current_record = Record::new();
                }
                continue;
            }

            // Check for EOF (End of File)
            if self.check_tag("EOF") {
                self.skip_tag("EOF")?;
                break;
            }

            // Parse a field
            let field = self.parse_field()?;
            current_record.add_field(field);
        }

        // Don't forget any trailing record without EOR
        if !current_record.fields.is_empty() {
            records.push(current_record);
        }

        Ok(records)
    }

    fn parse_field(&mut self) -> Result<Field> {
        let start_pos = self.pos;

        // Expect '<'
        if self.peek_char() != Some('<') {
            return Err(AdifError::ParseError {
                position: self.pos,
                message: "Expected '<' at start of field".to_string(),
            });
        }
        self.pos += 1;

        // Parse field name (until ':' or '>')
        let name_start = self.pos;
        while let Some(c) = self.peek_char() {
            if c == ':' || c == '>' {
                break;
            }
            self.pos += 1;
        }
        let name = self.input[name_start..self.pos].to_uppercase();

        if name.is_empty() {
            return Err(AdifError::InvalidDataSpecifier {
                position: start_pos,
                message: "Empty field name".to_string(),
            });
        }

        // Check if this is a marker tag (no length)
        if self.peek_char() == Some('>') {
            self.pos += 1;
            return Ok(Field::new(name, ""));
        }

        // Expect ':'
        if self.peek_char() != Some(':') {
            return Err(AdifError::InvalidDataSpecifier {
                position: self.pos,
                message: "Expected ':' after field name".to_string(),
            });
        }
        self.pos += 1;

        // Parse length
        let length_start = self.pos;
        while let Some(c) = self.peek_char() {
            if !c.is_ascii_digit() {
                break;
            }
            self.pos += 1;
        }

        let length_str = &self.input[length_start..self.pos];
        let length: usize = length_str
            .parse()
            .map_err(|_| AdifError::InvalidDataSpecifier {
                position: length_start,
                message: format!("Invalid length: '{}'", length_str),
            })?;

        // Check for optional type indicator
        let data_type = if self.peek_char() == Some(':') {
            self.pos += 1;
            let type_char = self
                .peek_char()
                .ok_or_else(|| AdifError::UnexpectedEof(self.pos))?;
            self.pos += 1;
            DataType::from_char(type_char).unwrap_or(DataType::Unspecified)
        } else {
            DataType::Unspecified
        };

        // Expect '>'
        if self.peek_char() != Some('>') {
            return Err(AdifError::InvalidDataSpecifier {
                position: self.pos,
                message: format!("Expected '>' to close tag, found {:?}", self.peek_char()),
            });
        }
        self.pos += 1;

        // Read the value (exactly 'length' characters)
        if self.pos + length > self.input.len() {
            return Err(AdifError::InvalidFieldLength {
                position: self.pos,
                expected: length,
                found: self.input.len() - self.pos,
            });
        }

        let value = self.input[self.pos..self.pos + length].to_string();
        self.pos += length;

        Ok(Field::with_type(name, data_type, value))
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn skip_whitespace_and_newlines(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
    }

    fn find_next_tag_start(&self) -> Option<usize> {
        self.input[self.pos..].find('<').map(|i| self.pos + i)
    }

    fn check_tag(&self, tag_name: &str) -> bool {
        let remaining = &self.input[self.pos..];
        if remaining.len() < tag_name.len() + 2 {
            return false;
        }

        // Check for <TAG_NAME> or <TAG_NAME: (with possible content after)
        let upper = remaining.to_uppercase();
        if !upper.starts_with('<') {
            return false;
        }

        let after_bracket = &upper[1..];
        if after_bracket.starts_with(tag_name) {
            let after_name = &after_bracket[tag_name.len()..];
            // Must be followed by '>' or ':'
            after_name.starts_with('>') || after_name.starts_with(':')
        } else {
            false
        }
    }

    fn skip_tag(&mut self, tag_name: &str) -> Result<()> {
        // Skip '<'
        if self.peek_char() != Some('<') {
            return Err(AdifError::ParseError {
                position: self.pos,
                message: format!("Expected '<' for {} tag", tag_name),
            });
        }
        self.pos += 1;

        // Skip tag name
        self.pos += tag_name.len();

        // Skip until '>'
        while let Some(c) = self.peek_char() {
            self.pos += c.len_utf8();
            if c == '>' {
                break;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_record() {
        let input = "<CALL:6>W1AW00<QSO_DATE:8>20240115<TIME_ON:6>143000<BAND:3>20m<MODE:2>CW<EOR>";
        let result = parse_adi(input).unwrap();

        assert_eq!(result.records.len(), 1);
        let record = &result.records[0];
        assert_eq!(record.call(), Some("W1AW00"));
        assert_eq!(record.qso_date(), Some("20240115"));
        assert_eq!(record.time_on(), Some("143000"));
        assert_eq!(record.band(), Some("20m"));
        assert_eq!(record.mode(), Some("CW"));
    }

    #[test]
    fn test_parse_with_header() {
        let input = r#"ADIF Export
<ADIF_VER:5>3.1.4
<PROGRAMID:4>Test
<EOH>
<CALL:6>N0CALL<QSO_DATE:8>20240115<EOR>
"#;
        let result = parse_adi(input).unwrap();

        assert_eq!(result.header.adif_version, Some("3.1.4".to_string()));
        assert_eq!(result.header.program_id, Some("Test".to_string()));
        assert!(result.header.preamble.contains("ADIF Export"));
        assert_eq!(result.records.len(), 1);
        assert_eq!(result.records[0].call(), Some("N0CALL"));
    }

    #[test]
    fn test_parse_multiple_records() {
        let input = "<CALL:5>W1AW1<EOR><CALL:5>W1AW2<EOR><CALL:5>W1AW3<EOR>";
        let result = parse_adi(input).unwrap();

        assert_eq!(result.records.len(), 3);
        assert_eq!(result.records[0].call(), Some("W1AW1"));
        assert_eq!(result.records[1].call(), Some("W1AW2"));
        assert_eq!(result.records[2].call(), Some("W1AW3"));
    }

    #[test]
    fn test_parse_with_type_indicator() {
        let input = "<FREQ:6:N>14.256<EOR>";
        let result = parse_adi(input).unwrap();

        assert_eq!(result.records.len(), 1);
        let freq_field = result.records[0].get("FREQ").unwrap();
        assert_eq!(freq_field.value, "14.256");
        assert_eq!(freq_field.data_type, DataType::Number);
    }

    #[test]
    fn test_case_insensitive_field_names() {
        let input = "<call:5>W1AW1<Call:5>W1AW2<CALL:5>W1AW3<EOR>";
        let result = parse_adi(input).unwrap();

        assert_eq!(result.records.len(), 1);
        // All three fields should be stored as uppercase
        let calls: Vec<&str> = result.records[0]
            .fields
            .iter()
            .filter(|f| f.name == "CALL")
            .map(|f| f.value.as_str())
            .collect();
        assert_eq!(calls.len(), 3);
    }

    #[test]
    fn test_parse_with_eof() {
        let input = "<CALL:5>W1AW1<EOR><EOF>";
        let result = parse_adi(input).unwrap();

        assert_eq!(result.records.len(), 1);
    }

    #[test]
    fn test_empty_file() {
        let input = "";
        let result = parse_adi(input).unwrap();

        assert!(result.records.is_empty());
    }

    #[test]
    fn test_header_only() {
        let input = "<ADIF_VER:5>3.1.4<EOH>";
        let result = parse_adi(input).unwrap();

        assert_eq!(result.header.adif_version, Some("3.1.4".to_string()));
        assert!(result.records.is_empty());
    }
}
