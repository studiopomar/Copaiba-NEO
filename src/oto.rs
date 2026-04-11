use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

/// Detected encoding of the oto.ini file
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum OtoEncoding {
    Utf8,
    ShiftJis,
    Gbk,
}

/// One entry (alias) in an oto.ini file
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OtoEntry {
    pub filename: String, // e.g. "あ.wav"
    pub alias: String,    // e.g. "- あ"
    /// Offset from start of WAV in milliseconds
    pub offset: f64,
    /// Consonant area (from offset, in ms)
    pub consonant: f64,
    /// Cutoff from end of WAV (negative ms) or from offset (positive ms)
    pub cutoff: f64,
    /// Preutterance from offset in ms
    pub preutter: f64,
    /// Overlap from offset in ms
    pub overlap: f64,
    /// Index into the original line order (for stable save)
    #[allow(dead_code)]
    pub line_index: usize,
    /// Marked as done by user
    pub done: bool,
    /// User annotations
    pub notes: String,
}

impl OtoEntry {
    /// Serialize back to one oto.ini line
    pub fn to_line(&self) -> String {
        format!(
            "{}={},{},{},{},{},{}",
            self.filename,
            self.alias,
            self.offset.round(),
            self.consonant.round(),
            self.cutoff.round(),
            self.preutter.round(),
            self.overlap.round()
        )
    }
}

pub struct ParsedOto {
    pub entries: Vec<OtoEntry>,
    pub encoding: OtoEncoding,
}

/// Parse an oto.ini file from disk.
/// Tries to detect encoding (UTF-8 or Shift-JIS) based on success and character validity.
pub fn parse_oto(path: &Path) -> Result<ParsedOto, String> {
    parse_oto_with_encoding(path, None)
}

pub fn parse_oto_with_encoding(path: &Path, force_encoding: Option<OtoEncoding>) -> Result<ParsedOto, String> {
    let bytes = fs::read(path).map_err(|e: std::io::Error| e.to_string())?;

    let (text, encoding) = if let Some(forced) = force_encoding {
        match forced {
            OtoEncoding::Utf8 => (String::from_utf8_lossy(&bytes).into_owned(), OtoEncoding::Utf8),
            OtoEncoding::ShiftJis => (encoding_rs::SHIFT_JIS.decode(&bytes).0.into_owned(), OtoEncoding::ShiftJis),
            OtoEncoding::Gbk => (encoding_rs::GBK.decode(&bytes).0.into_owned(), OtoEncoding::Gbk),
        }
    } else {
        // Detection logic
        let (decoded_utf8, had_errors_utf8) = match std::str::from_utf8(&bytes) {
            Ok(s) => (s.to_string(), false),
            Err(_) => (String::from_utf8_lossy(&bytes).into_owned(), true),
        };

        let (decoded_sjis, _, had_errors_sjis) = encoding_rs::SHIFT_JIS.decode(&bytes);

        if !had_errors_utf8 {
            (decoded_utf8, OtoEncoding::Utf8)
        } else if !had_errors_sjis {
            (decoded_sjis.into_owned(), OtoEncoding::ShiftJis)
        } else {
            // Both have errors, pick the one with fewer replacement characters
            let utf8_errors = decoded_utf8.chars().filter(|&c| c == '\u{FFFD}').count();
            let sjis_errors = decoded_sjis.chars().filter(|&c| c == '\u{FFFD}').count();

            if sjis_errors <= utf8_errors {
                (decoded_sjis.into_owned(), OtoEncoding::ShiftJis)
            } else {
                (decoded_utf8, OtoEncoding::Utf8)
            }
        }
    };

    let mut entries = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(entry) = parse_line(line, idx) {
            entries.push(entry);
        }
    }

    Ok(ParsedOto { entries, encoding })
}

fn parse_line(line: &str, idx: usize) -> Option<OtoEntry> {
    // Format: filename=alias,offset,consonant,cutoff,preutter,overlap
    let eq = line.find('=')?;
    let filename = line[..eq].to_string();
    let rest = &line[eq + 1..];

    let parts: Vec<&str> = rest.splitn(7, ',').collect();
    if parts.len() < 6 {
        return None;
    }

    let alias = parts[0].to_string();
    let offset = parts.get(1).and_then(|s| s.trim().parse::<f64>().ok()).unwrap_or(0.0);
    let consonant = parts.get(2).and_then(|s| s.trim().parse::<f64>().ok()).unwrap_or(0.0);
    let cutoff = parts.get(3).and_then(|s| s.trim().parse::<f64>().ok()).unwrap_or(0.0);
    let preutter = parts.get(4).and_then(|s| s.trim().parse::<f64>().ok()).unwrap_or(0.0);
    let overlap = parts.get(5).and_then(|s| s.trim().parse::<f64>().ok()).unwrap_or(0.0);

    Some(OtoEntry {
        filename,
        alias,
        offset,
        consonant,
        cutoff,
        preutter,
        overlap,
        line_index: idx,
        done: false,
        notes: String::new(),
    })
}

pub fn save_oto(entries: &[OtoEntry], path: &Path, encoding: OtoEncoding) -> Result<(), String> {
    use std::io::Write;

    let mut buffer = Vec::with_capacity(entries.len() * 100);
    for entry in entries {
        writeln!(buffer, "{}", entry.to_line()).map_err(|e| e.to_string())?;
    }

    let final_bytes: Vec<u8> = match encoding {
        OtoEncoding::Utf8 => buffer,
        OtoEncoding::ShiftJis => {
            let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(std::str::from_utf8(&buffer).unwrap_or(""));
            encoded.into_owned()
        }
        OtoEncoding::Gbk => {
            let (encoded, _, _) = encoding_rs::GBK.encode(std::str::from_utf8(&buffer).unwrap_or(""));
            encoded.into_owned()
        }
    };

    fs::write(path, final_bytes).map_err(|e: std::io::Error| e.to_string())
}
