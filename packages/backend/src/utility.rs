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
