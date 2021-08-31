use std::fmt;

use serde::{ Serialize, Serializer };
use serde_json;
use serde_json::json;

#[derive(Debug)]
pub struct QuoteError {
    pub symbol: String,
    pub message: String,
    pub detail: Option<String>,
    pub source: Option<Box<dyn std::error::Error>>
}

impl fmt::Display for QuoteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let val = serde_json::to_string_pretty(self).unwrap();
        write!(f, "{}", val)
    }
}

impl Serialize for QuoteError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let json = json!({
            "symbol": self.symbol,
            "detail": self.detail.as_ref().unwrap_or(&"".into()),
            "source": match self.source {
                Some(ref err) => err.to_string(),
                _ => "".into()
            }
        });

        json.serialize(serializer)
    }
}
