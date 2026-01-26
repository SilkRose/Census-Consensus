use crate::structs::{Chapter, Session};
use maud::{DOCTYPE, PreEscaped, html};

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

pub fn chapters_html(chapters: Vec<Chapter>) -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				h1 { "Chapters" }
					br;
					table {
						tr {
							th { "ID" }
							th { "Title" }
							th { "Vote Duration" }
							th { "Minutes Left" }
							th { "Fimfic Chapter ID" }
							th { "Intro Length" }
							th { "Outro Length" }
							th { "Chapter Order" }
							th { "Created" }
							th { "Edit" }
						}
						@for chapter in chapters.iter() {
							(chapter_table_row(chapter))
						}
					}
					br;
					button onclick = "window.location.href='/chapters/new';" { "New Chapter" }
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
			{ (value) }
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
			required
			{ (value) }
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
			{ (value) }
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
			required
			{ (value) }
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
			{ (value) }
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
			required
			{ (value) }
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

fn chapter_table_row(chapter: &Chapter) -> PreEscaped<String> {
	html! (
		tr {
			td { (chapter.id) }
			td { (chapter.title) }
			td { (chapter.vote_duration) }
			td { (chapter.minutes_left.unwrap_or_default()) }
			td { (chapter.fimfic_ch_id.unwrap_or_default()) }
			td { (chapter.intro_text.clone().map(|text| text.len()).unwrap_or_default()) }
			td { (chapter.outro_text.clone().map(|text| text.len()).unwrap_or_default()) }
			td { (chapter.chapter_order.unwrap_or_default()) }
			td { (chapter.date_created.format("%d/%m/%Y %H:%M")) }
			td { button onclick = (format!("window.location.href='/chapters/{}';", chapter.id)) { "Edit" } }
		}
	)
}
