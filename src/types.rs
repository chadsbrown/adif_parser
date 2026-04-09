use std::collections::HashMap;
use std::fmt::Write;

/// ADIF data type indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    /// Boolean (Y/N)
    Boolean,
    /// Number (decimal with optional decimal point)
    Number,
    /// Date (YYYYMMDD)
    Date,
    /// Time (HHMMSS or HHMM)
    Time,
    /// String (ASCII characters 32-126)
    String,
    /// Multiline string (ASCII with CR-LF)
    MultilineString,
    /// Enumeration
    Enumeration,
    /// Location (XDDD MM.MMM format)
    Location,
    /// International string (UTF-8, ADX only)
    IntlString,
    /// International multiline string (UTF-8, ADX only)
    IntlMultilineString,
    /// Unknown/unspecified type
    Unspecified,
}

impl DataType {
    /// Parse a data type indicator character
    pub fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_uppercase() {
            'B' => Some(DataType::Boolean),
            'N' => Some(DataType::Number),
            'D' => Some(DataType::Date),
            'T' => Some(DataType::Time),
            'S' => Some(DataType::String),
            'M' => Some(DataType::MultilineString),
            'E' => Some(DataType::Enumeration),
            'L' => Some(DataType::Location),
            'I' => Some(DataType::IntlString),
            'G' => Some(DataType::IntlMultilineString),
            _ => None,
        }
    }

    /// Convert to the type indicator character
    pub fn to_char(self) -> Option<char> {
        match self {
            DataType::Boolean => Some('B'),
            DataType::Number => Some('N'),
            DataType::Date => Some('D'),
            DataType::Time => Some('T'),
            DataType::String => Some('S'),
            DataType::MultilineString => Some('M'),
            DataType::Enumeration => Some('E'),
            DataType::Location => Some('L'),
            DataType::IntlString => Some('I'),
            DataType::IntlMultilineString => Some('G'),
            DataType::Unspecified => None,
        }
    }
}

/// A single ADIF field with name, optional type, and value
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// Field name (case-insensitive, stored uppercase)
    pub name: String,
    /// Data type indicator (if specified)
    pub data_type: DataType,
    /// Field value
    pub value: String,
}

impl Field {
    /// Create a new field
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into().to_uppercase(),
            data_type: DataType::Unspecified,
            value: value.into(),
        }
    }

    /// Serialize this field to ADI format: `<NAME:length[:type]>value`
    pub fn to_adi_string(&self) -> String {
        let len = self.value.len();
        match self.data_type.to_char() {
            Some(t) => format!("<{}:{}:{}>{}",  self.name, len, t, self.value),
            None => format!("<{}:{}>{}",  self.name, len, self.value),
        }
    }

    /// Create a new field with a specific data type
    pub fn with_type(
        name: impl Into<String>,
        data_type: DataType,
        value: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into().to_uppercase(),
            data_type,
            value: value.into(),
        }
    }
}

/// ADIF file header containing metadata
#[derive(Debug, Clone, Default)]
pub struct AdifHeader {
    /// ADIF version
    pub adif_version: Option<String>,
    /// Program that created the file
    pub program_id: Option<String>,
    /// Program version
    pub program_version: Option<String>,
    /// File creation timestamp
    pub created_timestamp: Option<String>,
    /// All header fields (including the above)
    pub fields: Vec<Field>,
    /// Any text before the first tag (comments, etc.)
    pub preamble: String,
}

impl AdifHeader {
    /// Get a header field by name (case-insensitive)
    pub fn get(&self, name: &str) -> Option<&Field> {
        let name_upper = name.to_uppercase();
        self.fields.iter().find(|f| f.name == name_upper)
    }

    /// Get a header field value by name (case-insensitive)
    pub fn get_value(&self, name: &str) -> Option<&str> {
        self.get(name).map(|f| f.value.as_str())
    }
}

/// A single QSO (contact) record
#[derive(Debug, Clone, Default)]
pub struct Record {
    /// Fields in this record
    pub fields: Vec<Field>,
}

impl Record {
    /// Create a new empty record
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// Add a field to the record
    pub fn add_field(&mut self, field: Field) {
        self.fields.push(field);
    }

    /// Serialize this record to ADI format (one line, terminated by `<EOR>`).
    pub fn to_adi_string(&self) -> String {
        let mut s = String::new();
        for field in &self.fields {
            let _ = write!(s, "{}", field.to_adi_string());
        }
        s.push_str("<EOR>\n");
        s
    }

    /// Get a field by name (case-insensitive)
    pub fn get(&self, name: &str) -> Option<&Field> {
        let name_upper = name.to_uppercase();
        self.fields.iter().find(|f| f.name == name_upper)
    }

    /// Get a field value by name (case-insensitive)
    pub fn get_value(&self, name: &str) -> Option<&str> {
        self.get(name).map(|f| f.value.as_str())
    }

    /// Convert to a HashMap for easier access
    pub fn to_map(&self) -> HashMap<String, String> {
        self.fields
            .iter()
            .map(|f| (f.name.clone(), f.value.clone()))
            .collect()
    }

    /// Get the call sign of the contacted station
    pub fn call(&self) -> Option<&str> {
        self.get_value("CALL")
    }

    /// Get the QSO date
    pub fn qso_date(&self) -> Option<&str> {
        self.get_value("QSO_DATE")
    }

    /// Get the time on (start time)
    pub fn time_on(&self) -> Option<&str> {
        self.get_value("TIME_ON")
    }

    /// Get the band
    pub fn band(&self) -> Option<&str> {
        self.get_value("BAND")
    }

    /// Get the frequency in MHz
    pub fn freq(&self) -> Option<&str> {
        self.get_value("FREQ")
    }

    /// Get the mode
    pub fn mode(&self) -> Option<&str> {
        self.get_value("MODE")
    }

    /// Get the RST sent
    pub fn rst_sent(&self) -> Option<&str> {
        self.get_value("RST_SENT")
    }

    /// Get the RST received
    pub fn rst_rcvd(&self) -> Option<&str> {
        self.get_value("RST_RCVD")
    }
}

/// A complete ADIF file with header and records
#[derive(Debug, Clone, Default)]
pub struct AdifFile {
    /// File header (may be empty if no header present)
    pub header: AdifHeader,
    /// QSO records
    pub records: Vec<Record>,
}

impl AdifFile {
    /// Create a new empty ADIF file
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the number of records in the file
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Check if the file has no records
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Iterate over records
    pub fn iter(&self) -> impl Iterator<Item = &Record> {
        self.records.iter()
    }

    /// Serialize the entire file to ADI format.
    pub fn to_adi_string(&self) -> String {
        let mut s = String::new();

        // Header
        if !self.header.preamble.is_empty() {
            s.push_str(&self.header.preamble);
            s.push('\n');
        }
        for field in &self.header.fields {
            let _ = write!(s, "{}", field.to_adi_string());
        }
        if !self.header.fields.is_empty() || !self.header.preamble.is_empty() {
            s.push_str("<EOH>\n");
        }

        // Records
        for record in &self.records {
            s.push_str(&record.to_adi_string());
        }

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_to_adi_string() {
        let f = Field::new("CALL", "W1AW");
        assert_eq!(f.to_adi_string(), "<CALL:4>W1AW");
    }

    #[test]
    fn field_with_type_to_adi_string() {
        let f = Field::with_type("FREQ", DataType::Number, "14.025");
        assert_eq!(f.to_adi_string(), "<FREQ:6:N>14.025");
    }

    #[test]
    fn record_to_adi_string() {
        let mut rec = Record::new();
        rec.add_field(Field::new("CALL", "K1ABC"));
        rec.add_field(Field::new("BAND", "20m"));
        let s = rec.to_adi_string();
        assert!(s.starts_with("<CALL:5>K1ABC<BAND:3>20m"));
        assert!(s.trim_end().ends_with("<EOR>"));
    }

    #[test]
    fn roundtrip_parse_write_parse() {
        let original = "<CALL:5>W1AW1<QSO_DATE:8>20240115<BAND:3>20m<MODE:2>CW<EOR>\n\
                         <CALL:5>N0UNX<QSO_DATE:8>20240116<BAND:3>40m<MODE:2>CW<EOR>\n";
        let parsed = crate::parse_adi(original).unwrap();
        let written = parsed.to_adi_string();
        let reparsed = crate::parse_adi(&written).unwrap();

        assert_eq!(parsed.records.len(), reparsed.records.len());
        for (a, b) in parsed.records.iter().zip(reparsed.records.iter()) {
            assert_eq!(a.call(), b.call());
            assert_eq!(a.qso_date(), b.qso_date());
            assert_eq!(a.band(), b.band());
            assert_eq!(a.mode(), b.mode());
        }
    }

    #[test]
    fn roundtrip_with_header() {
        let mut file = AdifFile::new();
        file.header.preamble = "Generated by clogger".to_string();
        file.header.fields.push(Field::new("ADIF_VER", "3.1.4"));
        file.header.fields.push(Field::new("PROGRAMID", "clogger"));

        let mut rec = Record::new();
        rec.add_field(Field::new("CALL", "K3LR"));
        rec.add_field(Field::new("BAND", "20m"));
        file.records.push(rec);

        let adi = file.to_adi_string();
        let reparsed = crate::parse_adi(&adi).unwrap();

        assert_eq!(reparsed.header.adif_version, Some("3.1.4".to_string()));
        assert_eq!(reparsed.header.program_id, Some("clogger".to_string()));
        assert_eq!(reparsed.records.len(), 1);
        assert_eq!(reparsed.records[0].call(), Some("K3LR"));
    }
}
