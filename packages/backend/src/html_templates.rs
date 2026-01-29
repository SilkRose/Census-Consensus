use crate::{
	structs::{Chapter, ChapterTableData, Session},
	utility::count_words,
};
use maud::{DOCTYPE, PreEscaped, html};
use pony::word_stats::word_count;

pub fn update_user_info_html() -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				form method = "post" action = "/update-user" {
					p { "You can only update your info once per hour." }
					br;
					button type = "submit" { "Update User Info" }
				}
			};
		};
	}
	.into()
}

pub fn update_user_role_html() -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				form method = "post" action = "/user-role" {
					label for = "id" { "User ID:" }
					br;
					(input_text_numeric_required("id", "id", 1, 8))
					br;
					label for = "role" { "User Role:" }
					br;
					input id = "voter" type = "radio" name = "role" value = "voter" required {}
					label for = "voter" { "Voter" }
					input id = "writer" type = "radio" name = "role" value = "writer" {}
					label for = "writer" { "Writer" }
					input id = "admin" type = "radio" name = "role" value = "admin" {}
					label for = "admin" { "Admin" }
					br;
					button type = "submit" { "Update User Role" }
				}
			};
		};
	}
	.into()
}

pub fn ban_user_html() -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				form method = "post" action = "/ban-user" {
					label for = "id" { "User ID:" }
					br;
					(input_text_numeric_required("id", "id", 1, 8))
					br;
					label for = "reason" { "Ban Reason:" }
					br;
					(textarea_required("reason", "reason", 8, 256))
					br;
					button type = "submit" { "Ban User" }
				}
			};
		};
	}
	.into()
}

pub fn user_feedback_html(
	private_feedback: Option<String>, public_feedback: Option<String>,
) -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				form method = "post" action = "/user-feedback" {
					label for = "public" { h3  { "Public Feedback" } }
					br;
					p style = "opacity: 80%" { "May appear in a future blog post about this event." }
					(textarea_value("public", "feedback_public", 1_000_000, &public_feedback.unwrap_or_default()))
					br;
					label for = "private" { h3  { "Private Feedback" } }
					br;
					p style = "opacity: 80%" { "Shared only with the developers and writers of this event." }
					(textarea_value("private", "feedback_private", 1_000_000, &private_feedback.unwrap_or_default()))
					br;
					button type = "submit" { "Submit Feedback" }
				}
			};
		};
	}
	.into()
}

pub fn sessions_html(sessions: Vec<Session>) -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				form method = "post" action = "/revoke-sessions" {
					h1 { "Sessions" }
					br;
					table {
						tr {
							th { "Revoke?" }
							th { "User Agent" }
							th { "Created" }
							th { "Last Seen" }
						}
						(session_table_row(&sessions[0], 0))
						@for (num, session) in sessions.iter().enumerate().skip(1) {
							(session_table_row(session, num))
						}
					}
					br;
					button type = "submit" { "Revoke Sessions" }
				}
			};
		};
	}
	.into()
}

pub fn new_chapter_html() -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				form method = "post" action = "/chapters/new" {
					h1 { "New Chapter" }
					br;
					@let name = "title";
					label for = (name) { "Title:" }
					br;
					(input_text_required(name, name, 8, 256))
					br;
					@let name = "vote_duration";
					label for = (name) { "Vote Duration:" }
					br;
					(input_number_required(name, name, 1, 100))
					br;
					button type = "submit" { "Create Chapter" }
				}
			};
		};
	}
	.into()
}

pub fn edit_chapter_html(chapter: Chapter) -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				form method = "post" action = (format!("/chapters/{}", chapter.id)) {
					h1 { "Edit Chapter" }
					br;
					@let name = "title";
					label for = (name) { "Title:" }
					br;
					(input_text_value_required(name, name, 8, 256, &chapter.title))
					br;
					@let name = "vote_duration";
					label for = (name) { "Vote Duration:" }
					br;
					(input_number_value_required(name, name, 1, 100, chapter.vote_duration))
					br;
					@let name = "intro_text";
					label for = (name) { "Intro:" }
					br;
					(textarea_value(name, name, 1_000_000, &chapter.intro_text.unwrap_or_default()))
					br;
					@let name = "outro_text";
					label for = (name) { "Outro:" }
					br;
					(textarea_value(name, name, 1_000_000, &chapter.outro_text.unwrap_or_default()))
					br;
					button type = "submit" { "Save Chapter" }
				}
			};
		};
	}
	.into()
}

// HTML components go below this comment:

fn input_text(id: &str, name: &str, min: u32, max: u32) -> PreEscaped<String> {
	html!	(
		input
			id = (id)
			type = "text"
			name = (name)
			minlength = (min)
			maxlength = (max)
			{}
	)
}

fn input_text_value(id: &str, name: &str, min: u32, max: u32, value: &str) -> PreEscaped<String> {
	html!	(
		input
			id = (id)
			type = "text"
			name = (name)
			minlength = (min)
			maxlength = (max)
			value = (value)
			{}
	)
}

fn input_text_required(id: &str, name: &str, min: u32, max: u32) -> PreEscaped<String> {
	html!	(
		input
			id = (id)
			type = "text"
			name = (name)
			minlength = (min)
			maxlength = (max)
			required {}
	)
}

fn input_text_value_required(
	id: &str, name: &str, min: u32, max: u32, value: &str,
) -> PreEscaped<String> {
	html!	(
		input
			id = (id)
			type = "text"
			name = (name)
			minlength = (min)
			maxlength = (max)
			value = (value)
			required {}
	)
}

fn input_number(id: &str, name: &str, min: u32, max: u32) -> PreEscaped<String> {
	html!	(
		input
			id = (id)
			type = "number"
			name = (name)
			inputmode = "numeric"
			pattern = r"\d*"
			min = (min)
			max = (max)
			{}
	)
}

fn input_number_value(id: &str, name: &str, min: u32, max: u32, value: i32) -> PreEscaped<String> {
	html!	(
		input
			id = (id)
			type = "number"
			name = (name)
			inputmode = "numeric"
			pattern = r"\d*"
			min = (min)
			max = (max)
			value = (value)
			{}
	)
}

fn input_number_value_option(
	id: &str, name: &str, min: u32, max: u32, value: Option<i32>,
) -> PreEscaped<String> {
	html!	(
		input
			id = (id)
			type = "number"
			name = (name)
			inputmode = "numeric"
			pattern = r"\d*"
			min = (min)
			max = (max)
			value = (value.map_or(String::default(), |v| v.to_string()))
			{}
	)
}

fn input_number_required(id: &str, name: &str, min: u32, max: u32) -> PreEscaped<String> {
	html!	(
		input
			id = (id)
			type = "number"
			name = (name)
			inputmode = "numeric"
			pattern = r"\d*"
			min = (min)
			max = (max)
			required {}
	)
}

fn input_number_value_required(
	id: &str, name: &str, min: u32, max: u32, value: i32,
) -> PreEscaped<String> {
	html!	(
		input
			id = (id)
			type = "number"
			name = (name)
			inputmode = "numeric"
			pattern = r"\d*"
			min = (min)
			max = (max)
			value = (value)
			required {}
	)
}

fn input_text_numeric(id: &str, name: &str, min: u32, max: u32) -> PreEscaped<String> {
	html!	(
		input
			id = (id)
			type = "text"
			name = (name)
			inputmode = "numeric"
			pattern = r"\d*"
			minlength = (min)
			maxlength = (max)
			{}
	)
}

fn input_text_numeric_value(
	id: &str, name: &str, min: u32, max: u32, value: &str,
) -> PreEscaped<String> {
	html!	(
		input
			id = (id)
			type = "text"
			name = (name)
			inputmode = "numeric"
			pattern = r"\d*"
			minlength = (min)
			maxlength = (max)
			value = (value)
			{}
	)
}

fn input_text_numeric_required(id: &str, name: &str, min: u32, max: u32) -> PreEscaped<String> {
	html!	(
		input
			id = (id)
			type = "text"
			name = (name)
			inputmode = "numeric"
			pattern = r"\d*"
			minlength = (min)
			maxlength = (max)
			required {}
	)
}

fn input_text_numeric_value_required(
	id: &str, name: &str, min: u32, max: u32, value: &str,
) -> PreEscaped<String> {
	html!	(
		input
			id = (id)
			type = "text"
			name = (name)
			inputmode = "numeric"
			pattern = r"\d*"
			minlength = (min)
			maxlength = (max)
			value = (value)
			required {}
	)
}

fn textarea(id: &str, name: &str, max: u32) -> PreEscaped<String> {
	html!	(
		textarea
			id = (id)
			type = "text"
			name = (name)
			maxlength = (max)
			{}
	)
}

fn textarea_value(id: &str, name: &str, max: u32, value: &str) -> PreEscaped<String> {
	html!	(
		textarea
			id = (id)
			type = "text"
			name = (name)
			maxlength = (max)
			{ (value) }
	)
}

fn textarea_required(id: &str, name: &str, min: u32, max: u32) -> PreEscaped<String> {
	html!	(
		textarea
			id = (id)
			type = "text"
			name = (name)
			minlength = (min)
			maxlength = (max)
			required {}
	)
}

fn textarea_value_required(
	id: &str, name: &str, min: u32, max: u32, value: &str,
) -> PreEscaped<String> {
	html!	(
		textarea
			id = (id)
			type = "text"
			name = (name)
			minlength = (min)
			maxlength = (max)
			required
			{ (value) }
	)
}

fn button_link(text: &str, endpoint: &str) -> PreEscaped<String> {
	html! (
		button onclick = (format!("window.location.href='{endpoint}';")) { (text) }
	)
}

fn button_disabled(text: &str) -> PreEscaped<String> {
	html! (
		button disabled { (text) }
	)
}

fn session_table_row(session: &Session, num: usize) -> PreEscaped<String> {
	html! (
		tr {
			td { input type = "checkbox" id = (num) name = (num) value = (session.token) {} }
			@if num == 0 {
				td { b { "(Active) " } (session.user_agent) }
			} @else {
				td { (session.user_agent) }
			}
			td { (session.date_created.format("%d/%m/%Y %H:%M")) }
			td { (session.last_seen.format("%d/%m/%Y %H:%M")) }
		}
	)
}

pub fn chapters_html(chapters: Vec<Chapter>, admin: bool) -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				h1 { "Chapters" }
				br;
				table {
					tr {
						th { "ID" } // done
						th { "Title" } // done
						th { "Chapter" br; "Number" } // done
						th { "Vote" br; "Duration" } // done
						th { "Minutes" br; "Left" } // done
						th { "Questions" }
						th { "Fimfic" br; "Ch ID" } // done
						th { "Intro" br; "Words" } // done
						th { "Outro" br; "Words" } // done
						th { "Revisions" }
						th { "Edit" } // done
						th { "Last" br; "Edit" } // done
						th { "Created" } // done
					}
					@let mut prev_published = false;
					@for chapter in chapters.into_iter() {
						(chapter_table_row(chapter, &mut prev_published, admin))
					}
				}
			};
		};
	}
	.into()
}

fn chapter_table_row(
	chapter: Chapter, prev_published: &mut bool, admin: bool,
) -> PreEscaped<String> {
	let first_number =
		chapter.fimfic_ch_id.is_none() && chapter.chapter_order.is_some() && !*prev_published;
	*prev_published = first_number && chapter.fimfic_ch_id.is_some();
	html! (
		tr {
			td { (chapter.id) }
			td { (chapter.title) }
			td {
				@if let Some(order) = chapter.chapter_order {
					@if chapter.fimfic_ch_id.is_none() && admin {
						@if !first_number {
						@let endpoint = format!("/chapters/{}/ordered/-1", chapter.id);
						(button_link("▲", &endpoint))
						} @else {
							(button_disabled("▲"))
						}
					}
					(order)
					@if chapter.fimfic_ch_id.is_none() && admin {
						@let endpoint = format!("/chapters/{}/ordered/1", chapter.id);
						(button_link("▼", &endpoint))
					}
				} @else {
					@let endpoint = format!("/chapters/{}/ordered", chapter.id);
					(button_link("Add", &endpoint))
				}
			}
			td {
				@if chapter.fimfic_ch_id.is_none() && admin {
					@let endpoint = format!("/chapters/{}/vote-duration/1", chapter.id);
					(button_link("▲", &endpoint))
				}
				(chapter.vote_duration)
				@if chapter.fimfic_ch_id.is_none() && admin {
					@let endpoint = format!("/chapters/{}/vote-duration/-1", chapter.id);
					(button_link("▼", &endpoint))
				}
			}
			td {
				@if let Some(minutes_left) = chapter.minutes_left {
					@if chapter.fimfic_ch_id.is_none() && admin {
						@let endpoint = format!("/chapters/{}/minutes-left/1", chapter.id);
						(button_link("▲", &endpoint))
					}
					(minutes_left)
					@if chapter.fimfic_ch_id.is_none() && admin {
						@let endpoint = format!("/chapters/{}/minutes-left/-1", chapter.id);
						(button_link("▼", &endpoint))
					}
				}
			}
			td {} // need to do questions
			td { (chapter.fimfic_ch_id.map_or(String::default(), |m| m.to_string())) }
			td { (chapter.intro_text.clone().map(|text| count_words(&text)).unwrap_or_default()) }
			td { (chapter.outro_text.clone().map(|text| count_words(&text)).unwrap_or_default()) }
			td {} // need to do revisions
			td { button onclick = (format!("window.location.href='/chapters/{}';", chapter.id)) { "Edit" } }
			td { (chapter.last_edit.format("%d/%m/%Y")) br; (chapter.last_edit.format("%H:%M")) }
			td { (chapter.date_created.format("%d/%m/%Y")) br; (chapter.date_created.format("%H:%M")) }
		}
	)
}
