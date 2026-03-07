use crate::structs::QuestionType;
use actix_web::HttpRequest;
use pony::word_stats::word_count;

pub fn redirect(req: HttpRequest) -> String {
	req.headers()
		.get("Referer")
		.and_then(|v| v.to_str().ok())
		.unwrap_or("/")
		.into()
}

pub fn count_words(text: &str) -> usize {
	let count = word_count(text);
	match count {
		Ok(count) => count,
		Err(_) => text.split_whitespace().count(),
	}
}

pub fn count_options(text: &str, question_type: QuestionType) -> u32 {
	let mut count = 0;
	for line in text.lines() {
		if question_type == QuestionType::Scale {
			if line.starts_with("[") {
				let line = line.replace("[", "").replace("]", "");
				let numbers = line.split_once("-");
				if let Some((start, end)) = numbers
					&& let Ok(start) = start.parse::<u32>()
					&& let Ok(end) = end.parse::<u32>()
				{
					return end - start + 1;
				} else {
					return count;
				}
			}
		} else if !line.is_empty() && !line.starts_with("//") && !line.starts_with("Order:") {
			count += 1
		}
	}
	count
}

pub fn count_outcomes(text: &str) -> u32 {
	let mut count = 0;
	for line in text.lines() {
		if !line.is_empty() && line.starts_with("# ") {
			count += 1
		}
	}
	count
}
