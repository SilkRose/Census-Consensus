#![allow(unused, reason = "todo remove me")]

use pest::Parser;

pub struct Vote<'h> {
	count: u64,
	text: &'h str
}

struct VoteWithPercentage<'h> {
	count: u64,
	text: &'h str,
	percentage: f64
}

enum SpecifiedOption {
	OptionLetter(char),
	OptionNumber(u32),
	Ordinal(u32)
}

pub fn format(
	input: &str,
	votes: &[Vote]
) -> (String, Vec<String>) {
	use result_parser::*;

	enum ParseState {
		None,
		Start,
		End,
		Matching
	}

	let total_count = votes.iter().map(|v| v.count).sum::<u64>();
	let total_count_f64 = total_count as f64;
	let votes = votes
		.iter()
		.map(|v| VoteWithPercentage {
			count: v.count,
			text: v.text,
			percentage: (v.count as f64 / total_count_f64) * 100.0
		}).collect::<Vec<_>>();
	let mut state = ParseState::None;
	let mut matched = false;
	let mut start = None;
	let mut end = None;
	let mut middle = String::new();
	let mut errors = Vec::new();

	macro_rules! current_match_mut {
		() => {
			match state {
				ParseState::Start => { start.as_mut().unwrap() }
				ParseState::End => { end.as_mut().unwrap() }
				ParseState::Matching => { &mut middle }
				ParseState::None => { unreachable!() }
			}
		}
	}

	let lines = match ResultParser::parse(Rule::result_parse, input) {
		Ok(lines) => { lines }
		Err(err) => {
			// can't parse I guess
			errors.push(err.to_string());
			return (input.into(), errors);
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

						state = ParseState::Start;
						start = Some(String::new());
					}

					Rule::cond_end => {
						if end.is_some() {
							state = ParseState::None;
							errors.push("got more than one `# END` conditions".into());
							continue;
						}

						state = ParseState::End;
						end = Some(String::new());
					}

					Rule::cond_option => {
						let Some(vote) = get_count_from_str(first.as_str(), &votes, &mut errors) else {
							state = ParseState::None;
							continue;
						};
						let vote = vote.percentage;

						let comparison = match pairs.next().unwrap().as_rule() {
							Rule::cond_comparison_gt => { f64::gt }
							_ => { unreachable!() }
						};

						let next = pairs.next().unwrap();
						let other_percent = match next.as_rule() {
							Rule::cond_option => {
								let Some(other_vote) = get_count_from_str(next.as_str(), &votes, &mut errors) else {
									state = ParseState::None;
									continue;
								};
								other_vote.percentage
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

							_ => { unreachable!() }
						};

						state = if comparison(&vote, &other_percent) {
							ParseState::Matching
						} else {
							ParseState::None
						}
					}

					_ => { unreachable!() }
				}
			}

			Rule::result_next_text => {
				let mut pairs = line.into_inner().peekable();

				while let Some(segment) = pairs.next() {
					let mut option = match segment.as_rule() {
						Rule::text_normal_text => {
							current_match_mut!().push_str(segment.as_str());
							continue;

						}

						Rule::text_option_question => {
							current_match_mut!().push_str("todo get the question text as input then put it here");
							continue;
						}

						Rule::text_option_letter => {
							SpecifiedOption::OptionLetter(segment.as_str().chars().next().unwrap())
						}

						Rule::text_option_number => {
							SpecifiedOption::OptionNumber(segment.as_str().parse().unwrap())
						}

						_ => { unreachable!() }
					};

					if matches!(pairs.peek().unwrap().as_rule(), Rule::text_vote_place_indicator) {
						pairs.next();
						if let SpecifiedOption::OptionNumber(place) = option {
							option = SpecifiedOption::Ordinal(place)
						}
					}

					let next = pairs.next().unwrap();
					if matches!(next.as_rule(), Rule::text_vote_count) {
						current_match_mut!().push_str(&format!("{total_count}"));
						continue;
					}

					let precision = pairs.peek().unwrap();
					let precision = matches!(precision.as_rule(), Rule::text_float_precision)
						.then(|| precision.as_str().parse().unwrap())
						.unwrap_or(0);

					match next.as_rule() {
						Rule::text_vote_percent => {
							// current_match_mut!().push_str(&format!("{:.precision$}"));
							// todo vote percent??? where do I get this data
						}

						Rule::text_vote_count_formatted => {
							current_match_mut!()
								.push_str(&format_count_words(total_count, precision));
						}

						_ => { unreachable!() }
					};
				}
			}

			Rule::result_next_comment => { /* ignore :3 */}
			Rule::EOI => { break }
			_ => { unreachable!() }
		}
	}

	(middle, errors)
}

fn format_count_words(
	count: u64,
	decimal_places: usize
) -> String {
	let words = [
		" thousand",
		" million",
		" billion",
		" trillion",
		// will we ever need more than this?
	];
	let mut count = count as f64;
	let mut word = "";

	for w in words {
		if (0.0..1000.0).contains(&count) { break }

		word = w;
		count /= 1000.0;
	}

	format!("{count:.decimal_places$}{word}")
}

fn get_count_from_str<'h>(
	str: &'_ str,
	votes: &'h [VoteWithPercentage<'h>],
	errors: &'_ mut Vec<String>
) -> Option<&'h VoteWithPercentage<'h>> {
	let index = map_option_to_array_index(str.chars().next().unwrap()).unwrap();
	let vote = votes.get(index);

	if vote.is_none() {
		errors.push(format!("{str} is not a valid option"));
	}

	vote
}

fn map_option_to_array_index(option: char) -> Option<usize> {
	let (i, _) = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
		.chars()
		.enumerate()
		.find(|(_, i)| *i == option)?;

	Some(i)
}

mod result_parser {
	use super::*;

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

		cond_option = { ASCII_ALPHA_UPPER }
		cond_percentage = { ASCII_DIGIT{,2} }
		cond_percentage_wrap = _{ cond_percentage ~ "%" }
		cond_fraction_part = { ASCII_DIGIT{1,5} }
		cond_fraction = { cond_fraction_part ~ "/" ~ cond_fraction_part }

		cond_option_ext = _{ cond_option | cond_percentage_wrap | cond_fraction }

		cond_condition = _{ cond_option ~ (cond_comparison ~ cond_option_ext)? ~ (cond_booleans ~ cond_condition)? }
		cond_partial = _{ cond_start | cond_end | cond_condition }
		cond = _{ SOI ~ cond_partial ~ EOI }
		cond_line = _{ "# " ~ cond_partial }


		// text (result text)
		text_normal_text_char = _{ !"%" ~ !nl_char ~ ANY }
		text_normal_text = { text_normal_text_char+ }

		text_float_precision = { ASCII_DIGIT }
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
