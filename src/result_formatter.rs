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

pub fn format(
	input: &str,
	votes: &[Vote]
) -> (String, Vec<String>) {
	enum ParseState {
		None,
		Start,
		End,
		Matching,
	}

	let total_count = votes.iter().map(|v| v.count).sum::<u64>();
	let votes = votes
		.iter()
		.map(|v| VoteWithPercentage {
			count: v.count,
			text: v.text,
			percentage: (v.count as f64 / total_count as f64) * 100.0
		}).collect::<Vec<_>>();
	let mut state = ParseState::None;
	let mut matched = false;
	let mut start = None;
	let mut end = None;
	let mut middle = String::new();
	let mut errors = Vec::new();

	let lines = input.lines()
		.filter(|line| !line.starts_with("//"))
		.map(str::trim);

	for line in lines {
		match line {
			"# START" if start.is_none() => {
				// haven't seen a start yet
				state = ParseState::Start;
				start = Some(String::new());
			}

			"# END" if end.is_none() => {
				// haven't seen an end yet
				state = ParseState::End;
				end = Some(String::new());
			}

			"# START" | "# END" => {
				// have seen start or end, ignore
				state = ParseState::None
			}

			line if line.starts_with("# ") && !matched && parse_condition(line[..2].trim(), &votes, total_count, &mut errors) => {
				// regular condition that matches and we haven't had a match yet
				// start matching
				state = ParseState::Matching;
				matched = true;
			}

			line if line.starts_with("# ") && line.len() > 2 && matches!(state, ParseState::Matching) => {
				// next condition after already matching, ignore regardless of match
				state = ParseState::None;
			}

			line if matches!(state, ParseState::Start) => {
				// regular line while matching start, process and add it
				let start = start.as_mut().unwrap();
				start.push('\n');
				start.push_str(&parse_normal_line(line, &votes, total_count));
			}
			line if matches!(state, ParseState::End) => {
				// regular line while matching end, process and add it
				let end = end.as_mut().unwrap();
				end.push('\n');
				end.push_str(&parse_normal_line(line, &votes, total_count));
			}
			line if matches!(state, ParseState::Matching) => {
				// regular line while matching, process and add it
				middle.push('\n');
				middle.push_str(&parse_normal_line(line, &votes, total_count));
			}

			_ => {
				// not matching, do nothing
			}
		}
	}

	if let Some(start) = start {
		let temp = middle;
		middle = String::new();
		middle.push_str(&start);
		middle.push_str(&temp);
	}

	if let Some(end) = end {
		middle.push_str(&end);
	}

	(middle, errors)
}

fn parse_condition(
	condition: &str,
	votes: &[VoteWithPercentage],
	total_count: u64,
	errors: &mut Vec<String>
) -> bool {
	use condition_parser::*;

	// ...
	if votes.is_empty() { return false }

	// todo remove
	println!("input: {condition}");

	let mut condition = match ConditionParser::parse(Rule::parse, condition) {
		Ok(result) => { result }
		Err(err) => {
			errors.push(err.to_string());
			return false;
		}
	};
	let mut result = true;

	loop {
		let option = condition.next().unwrap();
		if matches!(option.as_rule(), Rule::EOI) { break }

		let Some((option_index, option_data)) = process_option(option, votes, errors) else {
			return false;
		};

		let next = condition.next().unwrap();

		match next.as_rule() {
			Rule::EOI => {
				let most = 0;

				// for vote in votes
				// votes.iter().enumerate().for_each(|(i, vote)| {

				// });
				return true
			}
			Rule::and => {}
			Rule::comparison_gt => {}
			_ => { unreachable!() }
		}

		// let other = condition.next().unwrap();

		// let Some(percentage) = process_option(option, votes, errors) else {
		// 	return false;
		// };

		// let (other_percentage) = match other.as_rule() {
		// 	Rule::option => {
		// 		let Some((option_index, option_data)) = process_option(other, votes, errors) else {
		// 			return false;
		// 		};

		// 	}
		// 	Rule::percentage => {}
		// 	Rule::fraction => {}
		// 	_ => { unreachable!() }
		// };
		// match comparison.as_rule() {
		// 	Rule::comparison_gt => {}
		// 	_ => { unreachable!() }
		// }
	}

	println!("{:?}", condition.next().unwrap());
	println!("{:?}", condition.next().unwrap());
	println!("{:?}", condition.next().unwrap());
	println!("{:?}", condition.next().unwrap());
	println!("{:?}", condition.next().unwrap());
	println!("{:?}", condition.next().unwrap());
	println!("{:?}", condition.next().unwrap());
	println!("{:?}", condition.next().unwrap());
	println!("{:?}", condition.next().unwrap());
	println!("{:?}", condition.next().unwrap());

	// todo fix this
	true



	// let mut condition = condition.chars();

	// macro_rules! some_or_return {
	// 	($var:ident = $expression:expr; else $msg:expr) => {
	// 		let Some($var) = $expression else {
	// 			errors.push($msg);
	// 			return false;
	// 		};
	// 	}
	// }
	// some_or_return!(a = condition.next(); else "empty condition".into());
	// some_or_return!(a_index = map_option_to_array_index(a); else format!("{a} is not a valid option"));
	// some_or_return!(a_data = votes.get(a_index); else format!("{a} option doesn't exist"));
}

fn process_option<'h>(
	option: pest::iterators::Pair<condition_parser::Rule>,
	votes: &'h [VoteWithPercentage<'h>],
	errors: &mut Vec<String>
) -> Option<(usize, &'h VoteWithPercentage<'h>)> {
	debug_assert!(
		matches!(option.as_rule(), condition_parser::Rule::option),
		"passed non option into process_option (this is a bug)"
	);

	let option_index = option.as_str().chars().next().unwrap();
	let option_index = map_option_to_array_index(option_index).unwrap();

	let Some(option_data) = votes.get(option_index) else {
		errors.push(format!("{option} option doesn't exist"));
		return None;
	};

	Some((option_index, option_data))
}

fn parse_normal_line(line: &str, votes: &[VoteWithPercentage], total_count: u64) -> String {
	todo!()
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

fn map_option_to_array_index(option: char) -> Option<usize> {
	let (i, _) = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
		.chars()
		.enumerate()
		.find(|(_, i)| *i == option)?;

	Some(i)
}

mod condition_parser {
	use super::*;

	#[derive(pest_derive::Parser)]
	#[grammar_inline = r#"
		and = { " AND " }
		comparison_gt = { " > " }
		comparison = _{ comparison_gt }

		option = { ASCII_ALPHA_UPPER }
		percentage = { ASCII_DIGIT{,2} ~ "%" }
		fraction = { ASCII_DIGIT+ ~ "/" ~ ASCII_DIGIT+ }

		option_ext = _{ option | percentage | fraction }

		condition = _{ option ~ (comparison ~ option_ext)? ~ (and ~ condition)? }
		parse = _{ SOI ~ condition ~ EOI }
	"#]
	pub struct ConditionParser;
}

mod result_code_parser {
	use super::*;

	#[derive(pest_derive::Parser)]
	#[grammar_inline = r#"
		normal_text_char = _{ !"%" ~ ANY }
		normal_text = { normal_text_char* }

		float_precision = { ASCII_DIGIT }
		float_precision_wrap = _{ "." ~ float_precision }

		vote_percent = { "vp" }
		vote_percent_wrap = _{ vote_percent ~ float_precision_wrap? }
		vote_count = { "vcc" }
		vote_count_formatted = { "vcw" }
		vote_count_formatted_wrap = _{ vote_count_formatted ~ float_precision_wrap? }
		vote_place_indicator = { "p-" }
		name = { "name" }

		option_question = { "%[question]%" }

		option_letter = { ASCII_ALPHA }
		option_number = { ASCII_DIGIT }
		option = _{ option_letter | option_number }

		inners = _{ vote_place_indicator? ~ (vote_percent_wrap | vote_count | vote_count_formatted_wrap | name) }

		options = _{ "%" ~ option? ~ "[" ~ inners ~ options_end }
		options_end = { "]%" }
		parse = _{ SOI ~ (normal_text ~ (option_question | options))* ~ normal_text? ~ EOI }
	"#]
	pub struct ResultCodeParser;
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
		cond_fraction = { ASCII_DIGIT+ ~ "/" ~ ASCII_DIGIT+ }

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

		text_options = _{ "%" ~ text_option? ~ "[" ~ text_inners ~ "]%" }
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
