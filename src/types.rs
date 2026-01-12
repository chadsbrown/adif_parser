use std::collections::HashMap;

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
}
