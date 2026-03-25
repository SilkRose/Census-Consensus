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
					"published": true
			  }
		 }
	})
}

pub fn story_json(id: i32, title: &str, short_description: &str, description: &str) -> Value {
	// Construct the json for story updates.
	json!({
		"data": {
			"id": id,
			"attributes": {
				"title": title,
				"description": parse(description.trim(), &WarningType::Quiet),
				"short_description": short_description
			}
		}
	})
}

pub fn story_json_completed(
	id: i32, title: &str, short_description: &str, description: &str,
) -> Value {
	// Construct the json for story updates.
	json!({
		"data": {
			"id": id,
			"attributes": {
				"title": title,
				"description": parse(description.trim(), &WarningType::Quiet),
				"short_description": short_description,
				"completion_status": "complete"
			}
		}
	})
}
