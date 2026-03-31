use crate::structs::{OptionData, QuestionDataOption};
use pest::Parser;
use pony::number_format::{ FormatType, format_number_unit_metric };

#[expect(
	clippy::single_char_add_str,
	reason = "doesn't matter, and it's theoretically more efficient anyways (microoptimisation yippee)"
)]
pub fn format(input: &QuestionDataOption) -> (String, Vec<String>) {
	use result_parser::*;

	macro_rules! unreachable {
		() => {
			return (
				input.data.result_writing.clone().unwrap_or_default(),
				vec!["entered unreachable code, blame meadowsys :3c".into()]
			)
		}
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
							continue
						}

						let first_str = first.as_str();
						let Some(vote) = get_count_from_str_maybe_ordinal(first_str, &votes, &votes_sorted, &mut errors) else {
							errors.push(format!("{first_str} is not a valid option"));
							state = ParseState::None;
							continue;
						};
						let vote_percent = vote.percent;

						let comparison = match pairs.next().map(|p| p.as_rule()) {
							Some(Rule::cond_comparison_gt) => f64::gt,
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
									get_count_from_str_maybe_ordinal(next.as_str(), &votes, &votes_sorted, &mut errors)
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

						Rule::text_option_letter => {
							SpecifiedOption::OptionLetter(segment.as_str().chars().next().unwrap())
						}

						Rule::text_option_number => {
							SpecifiedOption::OptionNumber(segment.as_str().parse().unwrap())
						}

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
							get_count_from_char(option, &votes, &mut errors)
						}
						SpecifiedOption::OptionNumber(option) => {
							get_count_from_index(option, &votes, &mut errors)
						}
						SpecifiedOption::Ordinal(option) => {
							get_count_from_index(option, &votes_sorted, &mut errors)
						}
					}) else {
						continue;
					};

					let next = pairs.next().unwrap();
					if matches!(next.as_rule(), Rule::text_vote_count) {
						current_match_mut!().push_str(&option.count.to_string());
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
							current.push_str(
								format!("{vp:.precision$}", vp = option.percent)
									.trim_end_matches('0')
									.trim_end_matches('.')
							);
							current.push_str("%");
						}

						Rule::text_vote_count_formatted => {
							current_match_mut!()
								.push_str(
									&format_number_unit_metric(option.count as _, FormatType::ShortScaleName, precision)
										// analysed the function, and there is no codepath
										// in which this function will return Err
										.unwrap()
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

enum SpecifiedOption {
	OptionLetter(char),
	OptionNumber(usize),
	Ordinal(usize),
}

fn get_count_from_str_maybe_ordinal<'h>(
	str: &str, votes: &[&'h OptionData], votes_sorted: &[&'h OptionData], errors: &mut Vec<String>
) -> Option<&'h OptionData> {
	let ordinal = str.parse();

	if let Ok(ordinal) = ordinal {
		get_count_from_index(ordinal, votes_sorted, errors)
	} else {
		get_count_from_str(str, votes, errors)
	}
}

fn get_count_from_str<'h>(
	str: &str, votes: &[&'h OptionData], errors: &mut Vec<String>,
) -> Option<&'h OptionData> {
	get_count_from_char(str.chars().next().unwrap(), votes, errors)
}

fn get_count_from_char<'h>(
	char: char, votes: &[&'h OptionData], errors: &mut Vec<String>,
) -> Option<&'h OptionData> {
	let index = map_option_to_array_index(char).unwrap();
	get_count_from_impl(&char.to_string(), index, votes, errors)
}

fn get_count_from_index<'h>(
	index: usize, votes: &[&'h OptionData], errors: &mut Vec<String>,
) -> Option<&'h OptionData> {
	get_count_from_impl(&index.to_string(), index, votes, errors)
}

fn get_count_from_impl<'h>(
	orig: &'_ str, index: usize, votes: &[&'h OptionData], errors: &mut Vec<String>,
) -> Option<&'h OptionData> {
	// `index` starts at 1, but slice indexes start at 0, so we subtract 1
	let vote = votes.get(index.saturating_sub(1));

	if vote.is_none() {
		errors.push(format!("{orig} is not a valid option"));
	}

	vote.copied()
}

fn map_option_to_array_index(option: char) -> Option<usize> {
	let (i, _) = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
		.chars()
		.enumerate()
		.find(|(_, i)| *i == option)?;

	Some(i)
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
		cond_comparison = _{ cond_comparison_gt }

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
		text_option_number = { ASCII_DIGIT }
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

// todo remove me lol
pub fn test_fn() {
	let lol = r#"
// Always start this way
# START
Twilight looked at Pinkie Pie. "This first question is about you."

"Oh," Pinkie Pie said.

// And always end this way
# END
"Okay, okay, enough of that," said Rainbow Dash.  "Let's move on."

// If 'absolutely' has over 1/2 of all votes:
# C > 1/2

// this would get replaced with 26 million, as an example.
"%A[vcw]% ponies voted that you are absolutely cute!" Twilight said.

// This would get replaced with a percentage, such as 50.2%.
"Wow, that's %A[vp.1]% of all of Equestria!"

// On the other hand, if A wins by a landslide, then we use the following.
# A > 90%

// This uses the question replacement.
Twilight said, "The question was *%[question]%*"

Pinkie smiled. "What were the options?"

"They were: %A[name]%, %B[name]%, %C[name]%."

"Which one won?"

"%A[name]% with over 90 of all votes!"
//                     %
//                     ^ this little bitch is causing issues

// Since both A and C have similar connotation, we can use the replacements to have them share a result.
// If C has more than half the votes or A has more than 90% of the votes, we will use the replacement text above and not consider this possibility.
// If either A has over 40% of the votes or C has over 40% of the votes, we use the following text.
// For example, we will use it if A, B, and C get 41%, 46%, and 13%, respectively.
# A > 40% OR C > 40%

// The first replacement would be replaced by 'yes' or 'absolutely', and the second would be the percentage, such as 43%.
"%1[p-name]% won with %1[p-vcw]% ponies voting that you are cute!" Twilight said.

// If all we care about is an option winning, we just list that option.
// This condition will not be considered if C has more than half the votes, A has more than 90% of the votes, or either A or C has more than 40% of the votes.
// If none of those happen, and if B is the winner, then we use the following text.
// For example, if A, B, and C get 34%, 45%, and 21%, respectively, we will use the following text.
# B

// this would get replaced with 26 million, as an example.
"%B[vcw]% ponies voted that you aren't cute!" Twilight said, shocked.

Pinkie frowned.

// We can also compare options directly.
// We will not consider this possibility if C has more than half the votes; A has more than 90% of the votes; either A or C has more than 40% of the votes; or B is the winner.
// If none of those happen, then we use the following replacement text if C has more votes than B and B has more votes than A.
// For example, we use this if A, B, and C get 29%, 32%, and 39%, respectively.
# C > B AND B > A

// this would get replaced with 26 million, as an example.
"%A[vcw]% ponies voted that you are absolutely cute!" Twilight said.

"Wow, what came in second?" Pinkie asked.

// This would be replaced with: no and 22.56%
"%B[name]% with %B[vp.2]% of ponies voting for it."

// Every outcome needs a result text.
// So far, we have text for the following possibilities: C has more than half the votes; A has more than 90% of the votes; either A or C has more than 40% of the votes; B is the winner; or C got more votes than B and B got more votes than A.
// It is still possible that none of those happened.  For example, if A, B, and C got 36%, 26%, and 38%, respectively; or if they got 35%, 33%, and 32%.
// One way to be careful and make sure every possibility is covered is to have a condition for every winner.
# A

"%A[vcw]% ponies think you're cute!" Twilight said.

// This last condition wraps up our possibilities
# C

"%A[vcw]% ponies think you're absolutely cute!" Twilight said.

	"#.trim();

	use result_parser::*;

	let result = ResultParser::parse(Rule::result_parse, lol).unwrap_or_else(|err| panic!("{err}"));

	// let max_len = result.clone().fold(0, |acc, curr| curr.as_str().len().max(acc));
	// result.for_each(|thing| {
	// 	let mut padded = String::from(thing.as_str());
	// 	let amount = max_len - padded.len();
	// 	padded.push_str("\":");
	// 	std::iter::repeat_n(" ", amount)
	// 		.for_each(|space| padded.push_str(space));
	// 	println!("\"{padded} {thing:?}");
	// });

	result.for_each(|thing| println!("{:?}: {thing}", thing.as_rule()));
}
