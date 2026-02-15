use crate::structs::{
	Chapter, ChapterData, ChapterRevision, ChapterTable, Question, QuestionData, QuestionRevision,
	QuestionTable, QuestionType, Session, User, UserType,
};
use crate::utility::count_words;
use maud::{DOCTYPE, PreEscaped, html};
use pony::number_format::format_number_u128;

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
					@let name = "intro_text";
					label for = (name) { "Intro:" }
					br;
					(textarea(name, name, 1_000_000))
					br;
					@let name = "outro_text";
					label for = (name) { "Outro:" }
					br;
					(textarea(name, name, 1_000_000))
					br;
					button type = "submit" { "Create Chapter" }
				}
			};
		};
	}
	.into()
}

pub fn edit_chapter_html(chapter: Chapter, data: ChapterRevision) -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				form method = "post" action = (format!("/chapters/{}", chapter.id)) {
					h1 { "Edit Chapter" }
					br;
					@let name = "title";
					label for = (name) { "Title:" }
					br;
					(input_text_value_required(name, name, 8, 256, &data.title))
					br;
					@let name = "intro_text";
					label for = (name) { "Intro:" }
					br;
					(textarea_value(name, name, 1_000_000, &data.intro_text.unwrap_or_default()))
					br;
					@let name = "outro_text";
					label for = (name) { "Outro:" }
					br;
					(textarea_value(name, name, 1_000_000, &data.outro_text.unwrap_or_default()))
					br;
					button type = "submit" { "Save Chapter" }
				}
			};
		};
	}
	.into()
}

pub fn chapters_html(chapters: Vec<ChapterTable>, admin: bool) -> String {
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
						th { "Questions" } // done
						th { "Fimfic" br; "Ch ID" } // done
						th { "Intro" br; "Words" } // done
						th { "Outro" br; "Words" } // done
						th { "Revisions" } // done
						th { "Last" br; "Edit" } // done
						th { "Last" br; "Revision" } // done
						th { "Created" } // done
						th { "Edit" } // done
					}
					@let mut prev_published: Option<bool> = None;
					@for chapter in chapters.iter() {
						(chapter_table_row(chapter, &mut prev_published, admin))
					}
				}
				(button_link("New Chapter", "/chapters/new"))
			};
		};
	}
	.into()
}

fn chapter_table_row(
	chapter: &ChapterTable, prev_published: &mut Option<bool>, admin: bool,
) -> PreEscaped<String> {
	let active = chapter.meta.fimfic_ch_id.is_some() || chapter.meta.minutes_left.is_some();
	let first_number = !active && chapter.meta.chapter_order.is_some() && prev_published.is_none();
	*prev_published = match first_number {
		true => Some(chapter.meta.fimfic_ch_id.is_some()),
		false => None,
	};
	html! (
		tr {
			td { (chapter.meta.id) }
			td { (chapter.last_data.title) }
			td {
				@if let Some(order) = chapter.meta.chapter_order {
					@if !active && admin {
						@if !first_number {
						@let endpoint = format!("/chapters/{}/ordered/-1", chapter.meta.id);
						(button_link("▲", &endpoint))
						} @else {
							(button_disabled("▲"))
						}
					}
					(order)
					@if !active && admin {
						@let endpoint = format!("/chapters/{}/ordered/1", chapter.meta.id);
						(button_link("▼", &endpoint))
					}
				} @else {
					@let endpoint = format!("/chapters/{}/ordered", chapter.meta.id);
					(button_link("Add", &endpoint))
				}
			}
			td {
				@if chapter.meta.fimfic_ch_id.is_none() && admin {
					@let endpoint = format!("/chapters/{}/vote-duration/1", chapter.meta.id);
					(button_link("▲", &endpoint))
				}
				(chapter.meta.vote_duration)
				@if chapter.meta.fimfic_ch_id.is_none() && admin {
					@let endpoint = format!("/chapters/{}/vote-duration/-1", chapter.meta.id);
					(button_link("▼", &endpoint))
				}
			}
			td {
				@if let Some(minutes_left) = chapter.meta.minutes_left {
					@if chapter.meta.fimfic_ch_id.is_none() && admin {
						@let endpoint = format!("/chapters/{}/minutes-left/1", chapter.meta.id);
						(button_link("▲", &endpoint))
					}
					(minutes_left)
					@if chapter.meta.fimfic_ch_id.is_none() && admin {
						@let endpoint = format!("/chapters/{}/minutes-left/-1", chapter.meta.id);
						(button_link("▼", &endpoint))
					}
				}
			}
			td { (chapter.questions) }
			td { (chapter.meta.fimfic_ch_id.map_or(String::default(), |m| m.to_string())) }
			td { (chapter.last_data.intro_text.clone().map(|text| count_words(&text)).unwrap_or_default()) }
			td { (chapter.last_data.outro_text.clone().map(|text| count_words(&text)).unwrap_or_default()) }
			td {
				(chapter.revisions)
				button onclick = (format!("window.location.href='/chapters/{}/revisions';", chapter.meta.id)) { "View" }
			}
			td { (chapter.meta.last_edit.format("%d/%m/%Y %H:%M")) }
			td {
				(chapter.last_data.date_created.format("%d/%m/%Y %H:%M")) br;
				@if let Some(pfp_url) = &chapter.last_user.pfp_url {
					img src = (format!("{pfp_url}-32")) alt = (chapter.last_user.name) {}
					" - "
				}
				(chapter.last_user.name)
			}
			td {
				(chapter.first_data.date_created.format("%d/%m/%Y %H:%M")) br;
				@if let Some(pfp_url) = &chapter.first_user.pfp_url {
					img src = (format!("{pfp_url}-32")) alt = (chapter.first_user.name) {}
					" - "
				}
				(chapter.first_user.name)
			}
			td { button onclick = (format!("window.location.href='/chapters/{}';", chapter.meta.id)) { "Edit" } }
		}
	)
}

pub fn chapter_history_html(chapter: ChapterData) -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				h1 { "Chapter Revisions" }
				br;
				@for revision in chapter.data.into_iter() {
						details {
							summary {
								"Date: " (revision.date_created.format("%d/%m/%Y %H:%M"))
								" By: "
								@let user = chapter.users.get(&revision.id).expect("User will always be present.");
								@if let Some(pfp_url) = &user.pfp_url {
									img src = (format!("{pfp_url}-32")) alt = (user.name) {}
									" - "
								}
								(user.name)
							}
							"title: " (revision.title) br;
							"Intro:" br;
							(revision.intro_text.unwrap_or_default()) br;
							"outro:" br;
							(revision.outro_text.unwrap_or_default())
						}
					}
			};
		};
	}
	.into()
}

pub fn new_question_html() -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				form method = "post" action = "/questions/new" {
					h1 { "New Question" }
					br;
					@let name = "question_text";
					label for = (name) { "Question:" }
					br;
					(input_text_required(name, name, 8, 256))
					br;
					@let name = "question_type";
					label for = (name) { "Question Type: " }
					select name = (name) id = (name) {
						option value = "multiple_choice" { "Multiple Choice" }
						option value = "multi_select" { "Multi-Select" }
						option value = "scale" { "Scale" }
					}
					br;
					@let name = "response_percent";
					label for = (name) { "Response Percentage:" }
					br;
					(input_float_required(name, name, 0.0, 100.0, 0.01))
					br;
					@let name = "asked_by";
					label for = (name) { "Asked by:" }
					br;
					(input_text_required(name, name, 8, 256))
					br;
					@let name = "option_writing";
					label for = (name) { "Options:" }
					br;
					(textarea(name, name, 1_000_000))
					br;
					@let name = "result_writing";
					label for = (name) { "Result Writings:" }
					br;
					(textarea(name, name, 1_000_000))
					br;
					button type = "submit" { "Create Question" }
				}
			};
		};
	}
	.into()
}

pub fn edit_question_html(question: Question, data: QuestionRevision, population: u32) -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				form method = "post" action = (format!("/questions/{}", question.id)) {
					h1 { "New Question" }
					br;
					@let name = "question_text";
					label for = (name) { "Question:" }
					br;
					(input_text_value_required(name, name, 8, 256, &data.question_text))
					br;
					@let name = "question_type";
					label for = (name) { "Question Type: " }
					select name = (name) id = (name) {
						(question_type_match(data.question_type))
					}
					br;
					@let name = "response_percent";
					label for = (name) { "Response Percentage:" }
					br;
					(input_float_value_required(name, name, 0.0, 100.0, 0.1, data.response_percent))
					br;
					"Ponies answered: "
						(format_number_u128((population as f64 * data.response_percent / 100.0).round() as u128).unwrap())
						" out of: "
						(format_number_u128(population as u128).unwrap())
					" -- Updated on refresh."
					br;
					@let name = "asked_by";
					label for = (name) { "Asked by:" }
					br;
					(input_text_value_required(name, name, 8, 256, &data.asked_by))
					br;
					@let name = "option_writing";
					label for = (name) { "Options:" }
					br;
					(textarea_value(name, name, 1_000_000, &data.option_writing.unwrap_or_default()))
					br;
					@let name = "result_writing";
					label for = (name) { "Result Writings:" }
					br;
					(textarea_value(name, name, 1_000_000, &data.result_writing.unwrap_or_default()))
					br;
					button type = "submit" { "Save Question" }
				}
			};
		};
	}
	.into()
}

fn question_type_match(question_type: QuestionType) -> PreEscaped<String> {
	html!(
		@match question_type {
			 QuestionType::MultipleChoice => {
				option value = "multiple_choice" selected { "Multiple Choice" }
				option value = "multi_select" { "Multi-Select" }
				option value = "scale" { "Scale" }
			 },
			 QuestionType::Multiselect => {
				option value = "multiple_choice" { "Multiple Choice" }
				option value = "multi_select" selected { "Multi-Select" }
				option value = "scale" { "Scale" }
			 },
			 QuestionType::Scale => {
				option value = "multiple_choice" { "Multiple Choice" }
				option value = "multi_select" { "Multi-Select" }
				option value = "scale" selected { "Scale" }
			 },
		}
	)
}

pub fn question_history_html(question: QuestionData, population: u32) -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				h1 { "Question Revisions" }
				br;
				@for revision in question.data.into_iter() {
						details {
							summary {
								"Date: " (revision.date_created.format("%d/%m/%Y %H:%M"))
								" By: "
								@let user = question.users.get(&revision.id).expect("User will always be present.");
								@if let Some(pfp_url) = &user.pfp_url {
									img src = (format!("{pfp_url}-32")) alt = (user.name) {}
									" - "
								}
								(user.name)
							}
							"Question: " (revision.question_text) br;
							"Question Type: " (revision.question_type) br;
							"Response percent: " (revision.response_percent) br;
							"Ponies answered: "
								(format_number_u128((population as f64 * revision.response_percent / 100.0).round() as u128).unwrap())
								" out of: "
								(format_number_u128(population as u128).unwrap())
							br;
							"Asked By: " (revision.asked_by) br;
							"Options:" br;
							(revision.option_writing.unwrap_or_default()) br;
							"Results:" br;
							(revision.result_writing.unwrap_or_default())
						}
					}
			};
		};
	}
	.into()
}

pub fn chapter_questions_html(
	questions: Vec<QuestionTable>, chapter_id: i32, population: u32, user: User,
) -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				h1 { "Chapter Questions" }
				br;
				table {
					tr {
						th { "ID" } // done
						th { "Question" } // done
						th { "Question" br; "Type" } // done
						th { "Response" br; "Percent" } // done
						th { "Chapter" br; "Order" } // done
						th { "Options" } // done?
						th { "Outcomes" } // done?
						th { "Revisions" } // done
						th { "Claiment" } // done
						th { "Last" br; "Edit" }
						th { "Last" br; "Revision" }
						th { "Created" }
						th { "Edit" }
					}
					@for question in questions.into_iter() {
						(chapter_questions_table(question, chapter_id, population, &user))
					}
				}
				(button_link("New Question", "/questions/new"))
			};
		};
	}
	.into()
}

pub fn chapter_questions_table(
	question: QuestionTable, chapter_id: i32, population: u32, user: &User,
) -> PreEscaped<String> {
	html! {
		tr {
			td { (question.meta.id) }
			td { (question.last_data.question_text) }
			td { (question.last_data.question_type) }
			td {
				(format_number_u128((population as f64 * question.last_data.response_percent / 100.0).round() as u128).unwrap())
			}
			td {
				@if let Some(order) = question.meta.chapter_order {
					@if order > 1 {
						@let endpoint = format!("/chapters/{chapter_id}/questions/{}/ordered/-1", question.meta.id);
						(button_link("▲", &endpoint))
					} @else {
						(button_disabled("▲"))
					}
					(order)
					@let endpoint = format!("/chapters/{chapter_id}/questions/{}/ordered/1", question.meta.id);
					(button_link("▼", &endpoint))
				} @else {
					@let endpoint = format!("/chapters/{chapter_id}/questions/{}/ordered", question.meta.id);
					(button_link("Add", &endpoint))
				}
			}
			td { (question.options) }
			td { (question.outcomes) }
			td {
				(question.revisions)
				button onclick = (format!("window.location.href='/questions/{}/revisions';", question.meta.id)) { "View" }
			}
			td {
				@if let Some(claiment) = question.claiment {
					@if let Some(pfp_url) = &claiment.pfp_url {
						img src = (format!("{pfp_url}-32")) alt = (claiment.name) {}
						" - "
					}
				(user.name)
				@if user.id == claiment.id {
					@let endpoint = format!("/questions/{}/unclaim", question.meta.id);
					br; (button_link("Un-Claim", &endpoint))
				} @else if user.user_type == UserType::Admin {
					@let endpoint = format!("/questions/{}/unclaim", question.meta.id);
					br; (button_link("Revoke", &endpoint))
				}
				} @ else {
					@let endpoint = format!("/questions/{}/claim", question.meta.id);
					(button_link("Claim", &endpoint))
				}
			}
			td { (question.meta.last_edit.format("%d/%m/%Y %H:%M")) }
			td {
				(question.last_data.date_created.format("%d/%m/%Y %H:%M")) br;
				@if let Some(pfp_url) = &question.last_user.pfp_url {
					img src = (format!("{pfp_url}-32")) alt = (question.last_user.name) {}
					" - "
				}
				(question.last_user.name)
			}
			td {
				(question.first_data.date_created.format("%d/%m/%Y %H:%M")) br;
				@if let Some(pfp_url) = &question.first_user.pfp_url {
					img src = (format!("{pfp_url}-32")) alt = (question.first_user.name) {}
					" - "
				}
				(question.first_user.name)
			}
			td { button onclick = (format!("window.location.href='/questions/{}';", question.meta.id)) { "Edit" } }
		}
	}
}

// HTML components go below this comment:

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

fn input_float_required(id: &str, name: &str, min: f64, max: f64, step: f64) -> PreEscaped<String> {
	html!	(
		input
			id = (id)
			type = "number"
			name = (name)
			inputmode = "decimal"
			min = (min)
			max = (max)
			step = (step)
			required {}
	)
}

fn input_float_value_required(
	id: &str, name: &str, min: f64, max: f64, step: f64, value: f64,
) -> PreEscaped<String> {
	html!	(
		input
			id = (id)
			type = "number"
			name = (name)
			inputmode = "decimal"
			min = (min)
			max = (max)
			step = (step)
			value = (value)
			required {}
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
