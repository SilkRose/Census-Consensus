use pony::markdown::{WarningType, bbcode::parse};
use serde_json::{Value, json};

pub fn chapter_json(title: &str, content: &str, authors_note: Option<&str>) -> Value {
	// Construct the json for chapters.
	json!({
		 "data": {
			  "type": "chapter",
			  "attributes": {
					"title": title,
					"content": parse(content.trim(), &WarningType::Quiet),
					"authors_note": authors_note.unwrap_or_default(),
			  }
		 }
	})
}
