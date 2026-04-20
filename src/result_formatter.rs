use crate::structs::{OptionData, QuestionDataOption};
use pest::Parser;
use pony::number_format::{FormatType, format_number_u128, format_number_unit_metric};

#[expect(
	clippy::single_char_add_str,
	reason = "doesn't matter, and it's theoretically more efficient anyways (microoptimisation yippee)"
)]
pub fn format(input: &QuestionDataOption) -> (String, Vec<String>) {
	use result_parser::*;

	#[track_caller]
	fn gen_unreachable_message() -> String {
		let loc = core::panic::Location::caller();

		let file = loc.file();
		let line = loc.line();
		let column = loc.column();

		format!("entered unreachable code, blame meadowsys :3c (in {file}, at {line}:{column})")
	}

	macro_rules! unreachable {
		() => {
			return (
				input.data.result_writing.clone().unwrap_or_default(),
				vec![gen_unreachable_message()],
			)
		};
	}

	let input_str = input.data.result_writing.as_deref().unwrap_or_default();
	let votes = input.options.iter().collect::<Vec<_>>();
	let votes_sorted = {
		let mut votes_sorted = votes.clone();
		votes_sorted.sort_by_key(|v| (core::cmp::Reverse(v.count), v.order));
		votes_sorted
	};

	let mut state = ParseState::None;
	let mut start = None;
	let mut end = None;
	let mut middle = None;
	let mut errors = Vec::new();

	macro_rules! current_match_mut {
		() => {
			match state {
				ParseState::Start => start.as_mut().unwrap(),
				ParseState::End => end.as_mut().unwrap(),
				ParseState::Matching => middle.as_mut().unwrap(),
				ParseState::None => {
					unreachable!()
				}
			}
		};
	}

	let lines = match ResultParser::parse(Rule::result_parse, input_str) {
		Ok(lines) => lines,
		Err(err) => {
			// can't parse I guess
			errors.push(err.to_string());
			return (input_str.into(), errors);
		}
	};

	for line in lines {
		match line.as_rule() {
			Rule::result_next_condition => {
				let mut pairs = line.into_inner();

				let first = pairs.next().unwrap();
				match first.as_rule() {
					Rule::cond_start => {
						if start.is_some() {
							state = ParseState::None;
							errors.push("got more than one `# START` conditions".into());
							continue;
						}

						start = Some(String::new());
						state = ParseState::Start;
					}

					Rule::cond_end => {
						if end.is_some() {
							state = ParseState::None;
							errors.push("got more than one `# END` conditions".into());
							continue;
						}

						end = Some(String::new());
						state = ParseState::End;
					}

					Rule::cond_option => {
						if middle.is_some() {
							state = ParseState::None;
							continue;
						}

						let first_str = first.as_str();
						let Some(vote) = get_count_from_id(first_str, &votes, &mut errors) else {
							errors.push(format!("{first_str} is not a valid option"));
							state = ParseState::None;
							continue;
						};
						let vote_percent = vote.percent;

						let comparison = match pairs.next().map(|p| p.as_rule()) {
							Some(Rule::cond_comparison_gt) => f64::gt,
							Some(Rule::cond_comparison_lt) => f64::lt,
							None => {
								// we got a vote out, which means that thare are votes at all,
								// so indexing 0 won't panic
								if *votes_sorted[0].id == *vote.id {
									middle = Some(String::new());
									state = ParseState::Matching;
								} else {
									state = ParseState::None
								};

								continue;
							}
							Some(_) => {
								unreachable!()
							}
						};

						let next = pairs.next().unwrap();
						let other_percent = match next.as_rule() {
							Rule::cond_option => {
								let Some(other_vote) =
									get_count_from_id(dbg!(next.as_str()), &votes, &mut errors)
								else {
									errors.push(format!("{next} is not a valid option"));
									state = ParseState::None;
									continue;
								};
								other_vote.percent
							}

							Rule::cond_percentage => {
								(next.as_str().parse::<u64>().unwrap() as f64) / 100.0
							}

							Rule::cond_fraction => {
								let mut iter = next.into_inner();

								let frac1 = iter.next().unwrap();
								let frac2 = iter.next().unwrap();

								debug_assert!(matches!(frac1.as_rule(), Rule::cond_fraction_part));
								debug_assert!(matches!(frac2.as_rule(), Rule::cond_fraction_part));

								let frac1 = frac1.as_str().parse::<u64>().unwrap() as f64;
								let frac2 = frac2.as_str().parse::<u64>().unwrap() as f64;

								frac1 / frac2
							}

							_ => {
								unreachable!()
							}
						};

						if comparison(&vote_percent, &other_percent) {
							middle = Some(String::new());
							state = ParseState::Matching;
						} else {
							state = ParseState::None;
						}
					}

					_ => {
						unreachable!()
					}
				}
			}

			Rule::result_next_text => {
				if matches!(state, ParseState::None) {
					continue;
				}
				let mut pairs = line.into_inner().peekable();

				current_match_mut!().push_str("\n\n");

				while let Some(segment) = pairs.next() {
					let mut option = match segment.as_rule() {
						Rule::text_normal_text => {
							current_match_mut!().push_str(segment.as_str());
							continue;
						}

						Rule::text_option_question => {
							current_match_mut!().push_str(&input.data.question_text);
							continue;
						}

						Rule::text_option_letter => SpecifiedOption::OptionLetter(segment.as_str()),

						Rule::text_option_number => SpecifiedOption::OptionNumber(segment.as_str()),

						_ => {
							unreachable!()
						}
					};

					if matches!(
						pairs.peek().unwrap().as_rule(),
						Rule::text_vote_place_indicator
					) {
						pairs.next();
						if let SpecifiedOption::OptionNumber(place) = option {
							option = SpecifiedOption::Ordinal(place)
						}
					}

					let Some(option) = (match option {
						SpecifiedOption::OptionLetter(option) => {
							get_count_from_id(option, &votes, &mut errors)
						}
						SpecifiedOption::OptionNumber(option) => {
							get_count_from_id(option, &votes, &mut errors)
						}
						SpecifiedOption::Ordinal(option) => {
							get_count_from_ordinal(option, &votes_sorted, &mut errors)
						}
					}) else {
						continue;
					};

					let next = pairs.next().unwrap();
					if matches!(next.as_rule(), Rule::text_vote_count) {
						current_match_mut!()
							// analysed the function, and there is no codepath
							// in which this function will return Err
							.push_str(&format_number_u128(option.count as u128).unwrap());
						continue;
					}

					let precision = pairs.peek().unwrap();
					let precision = if matches!(precision.as_rule(), Rule::text_float_precision) {
						let parsed = precision.as_str().parse().unwrap();
						pairs.next();
						parsed
					} else {
						0
					};

					match next.as_rule() {
						Rule::text_vote_percent => {
							let current = current_match_mut!();
							let percent = format!("{vp:.precision$}", vp = option.percent);
							let percent = match precision == 0 {
								true => &*percent,
								// guard exists because trim_end_matches('0') on "0" eats everything
								false if percent.contains(".") => {
									percent.trim_end_matches('0').trim_end_matches('.')
								}
								false => &*percent,
							};

							current.push_str(percent);
							current.push_str("%");
						}

						Rule::text_vote_count_formatted => {
							current_match_mut!().push_str(
								&format_number_unit_metric(
									option.count as _,
									FormatType::ShortScaleName,
									precision,
									true,
								)
								// analysed the function, and there is no codepath
								// in which this function will return Err
								.unwrap(),
							);
						}

						Rule::text_name => {
							current_match_mut!().push_str(&option.text);
						}

						_ => {
							unreachable!()
						}
					};
				}
			}

			Rule::result_next_comment => { /* ignore :3 */ }
			Rule::EOI => break,
			_ => {
				unreachable!()
			}
		}
	}

	let mut all = start.unwrap_or_default();
	middle.inspect(|middle| all.push_str(middle));
	end.inspect(|end| all.push_str(end));

	(all, errors)
}

enum ParseState {
	None,
	Start,
	End,
	Matching,
}

enum SpecifiedOption<'h> {
	OptionLetter(&'h str),
	OptionNumber(&'h str),
	Ordinal(&'h str),
}

fn get_count_from_id<'h>(
	id: &str, votes: &[&'h OptionData], errors: &mut Vec<String>,
) -> Option<&'h OptionData> {
	process_option(id, votes.iter().find(|v| *v.id == *id), errors)
}

fn get_count_from_ordinal<'h>(
	ordinal: &str, votes: &[&'h OptionData], errors: &mut Vec<String>,
) -> Option<&'h OptionData> {
	process_option(
		ordinal,
		ordinal
			.parse::<usize>()
			.ok()
			.and_then(|ordinal| votes.get(ordinal - 1)),
		errors,
	)
}

fn process_option<'h>(
	orig: &str, data: Option<&&'h OptionData>, errors: &mut Vec<String>,
) -> Option<&'h OptionData> {
	match data {
		None => {
			errors.push(format!("{orig} is not a valid option"));
			None
		}
		Some(vote) => Some(vote),
	}
}

mod result_parser {
	#[derive(pest_derive::Parser)]
	#[grammar_inline = r##"
		nl_char = _{ "\r" | "\n" }
		not_nl_char = _{ !nl_char ~ ANY }
		eat_ws_and_nl = _{ (nl_char | " ")* }


		// condition
		cond_start = { "START" }
		cond_end = { "END" }

		cond_and = { " AND " }
		cond_or = { " OR " }
		cond_booleans = _{ cond_and | cond_or }
		cond_comparison_gt = { " > " }
		cond_comparison_lt = { " < " }
		cond_comparison = _{ cond_comparison_gt | cond_comparison_lt }

		cond_option = { ASCII_ALPHA | ASCII_DIGIT+ }
		cond_percentage = { ASCII_DIGIT{,2} }
		cond_percentage_wrap = _{ cond_percentage ~ "%" }
		cond_fraction_part = { ASCII_DIGIT{1,5} }
		cond_fraction = { cond_fraction_part ~ "/" ~ cond_fraction_part }

		cond_option_ext = _{ cond_percentage_wrap | cond_fraction | cond_option }

		cond_condition = _{ cond_option ~ (cond_comparison ~ cond_option_ext)? ~ (cond_booleans ~ cond_condition)? }
		cond_partial = _{ cond_start | cond_end | cond_condition }
		cond = _{ SOI ~ cond_partial ~ EOI }
		cond_line = _{ "# " ~ cond_partial }


		// text (result text)
		text_normal_text_char = _{ !"%" ~ !nl_char ~ ANY }
		text_normal_text = { text_normal_text_char+ }

		text_float_precision = { ASCII_DIGIT+ }
		text_float_precision_wrap = _{ "." ~ text_float_precision }

		text_vote_percent = { "vp" }
		text_vote_percent_wrap = _{ text_vote_percent ~ text_float_precision_wrap? }
		text_vote_count = { "vcc" }
		text_vote_count_formatted = { "vcw" }
		text_vote_count_formatted_wrap = _{ text_vote_count_formatted ~ text_float_precision_wrap? }
		text_vote_place_indicator = { "p-" }
		text_name = { "name" }

		text_option_question = { "%[question]%" }

		text_option_letter = { ASCII_ALPHA }
		text_option_number = { ASCII_DIGIT+ }
		text_option = _{ text_option_letter | text_option_number }

		text_inners = _{ text_vote_place_indicator? ~ (text_vote_percent_wrap | text_vote_count | text_vote_count_formatted_wrap | text_name) }

		text_options = _{ "%" ~ text_option ~ "[" ~ text_inners ~ "]%" }
		text_all_options = _{ text_option_question | text_options }
		text_partial_1 = _{ text_all_options ~ (text_normal_text ~ text_all_options?)* }
		text_partial_2 = _{ text_normal_text ~ (text_all_options ~ text_normal_text?)* }
		text_partial = _{ text_partial_1 | text_partial_2 }
		text = _{ SOI ~ text_partial ~ EOI }


		// comment
		comment_text = { not_nl_char* }
		comment_line = { "//" ~ comment_text }


		// result
		result_is_comment = _{ &"//" }
		result_is_condition = _{ &"# " }
		result_is_text = _{ !result_is_comment ~ !result_is_condition ~ &not_nl_char }

		result_next_comment = { result_is_comment ~ comment_line }
		result_next_comment_wrap = _{ result_next_comment ~ eat_ws_and_nl }
		result_next_condition = { result_is_condition ~ cond_line }
		result_next_condition_wrap = _{ result_next_condition ~ eat_ws_and_nl }
		result_next_text = { result_is_text ~ text_partial }
		result_next_text_wrap = _{ result_next_text ~ eat_ws_and_nl }

		result_parse_partial = _{
			result_next_comment_wrap*
			~ (
				result_next_condition_wrap ~ result_next_comment_wrap*
				~ (result_next_text_wrap ~ result_next_comment_wrap*)+
			)+
		}
		result_parse = _{ SOI ~ result_parse_partial ~ EOI }
	"##]
	pub struct ResultParser;
}
