use std::collections::HashMap;

use crate::structs::{
	OptionData, OptionType, Question, QuestionDataOption, QuestionRevision, QuestionType,
};
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

pub fn parse_options(text: &str, question_type: &QuestionType) -> Vec<(String, String)> {
	let mut options = HashMap::new();
	for line in text.lines() {
		if *question_type == QuestionType::Scale {
			if line.starts_with("[") {
				let line = line.replace("[", "").replace("]", "");
				let numbers = line.split_once("-");
				if let Some((start, end)) = numbers
					&& let Ok(start) = start.parse::<u32>()
					&& let Ok(end) = end.parse::<u32>()
				{
					for i in start..=end {
						options.insert(i.to_string(), i.to_string());
					}
				} else {
					return Vec::new();
				}
			}
		} else if !line.is_empty()
			&& !line.starts_with("//")
			&& !line.starts_with("Order:")
			&& let Some((id, opt)) = line.split_once(": ")
		{
			options.insert(id.to_string(), opt.to_string());
		}
	}
	let mut options = options.into_iter().collect::<Vec<_>>();
	if QuestionType::Scale == *question_type {
		options.sort_by_key(|o| o.0.parse::<i32>().unwrap());
	} else {
		options.sort_by_key(|o| o.0.clone());
	}
	options
}

#[bon::builder]
pub fn construct_question_data(
	meta: Question, data: QuestionRevision, option_data: OptionType,
	option_texts: Vec<(String, String)>, population: i32,
) -> QuestionDataOption {
	let mut total_count = 0;
	let mut options = Vec::new();
	let binding = data.option_writing.clone().unwrap_or_default();
	let ordering = binding
		.lines()
		.find(|line| line.starts_with("Order:"))
		.unwrap_or_default()
		.trim_start_matches("Order:")
		.split(",")
		.map(|c| c.trim())
		.collect::<Vec<_>>();
	let ponies = population as f64 * data.response_percent / 100.0;
	for (id, text) in option_texts {
		let (percent, count) = match option_data {
			OptionType::Percent(ref percents) => {
				let percent = percents.get(&id).cloned().unwrap_or_default();
				let count = (ponies * percent / 100.0).round() as u32;
				(percent, count)
			}
			OptionType::Count((ref counts, total_votes)) => {
				let vote_count = counts.get(&id).cloned().unwrap_or_default();
				let percent = (vote_count as f64 / total_votes as f64) * 100.0;
				let count = (ponies * percent / 100.0).round() as u32;
				(percent, count)
			}
		};
		let percent = if percent.is_nan() { 0.0 } else { percent };
		let order = ordering
			.iter()
			.enumerate()
			.find(|(_, opt)| **opt == id)
			.map_or(0, |(i, _)| i) as u32;
		let data = OptionData {
			id,
			text,
			percent,
			count,
			order,
		};
		options.push(data);
		total_count += count;
	}
	QuestionDataOption {
		meta,
		total_count,
		data,
		options,
	}
}
