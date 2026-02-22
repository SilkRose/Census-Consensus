use crate::endpoints::MIN_USER_UPDATE_TIME;
use crate::structs::*;
use crate::theme::Theme;
use crate::utility::count_words;
use bon::builder;
use chrono::Utc;
use maud::{DOCTYPE, PreEscaped, html};
use pony::number_format::format_number_u128;
use pony::time::format_milliseconds;
use url::form_urlencoded;

const SITE_NAME: &str = "Census Consensus";
const SITE_LINK: &str = "https://survey.silkrose.dev";

#[builder]
fn html_builder(
	theme: &Theme, head: PreEscaped<String>, header: PreEscaped<String>, mane: PreEscaped<String>,
) -> String {
	let body_content = html! {
		header { (header) }
		main { (mane) }
	};
	let body = match theme {
		Theme::Light => html! {body class = "light" { (body_content) }},
		Theme::Dark => html! {body class = "dark" { (body_content) }},
		Theme::None => html! {body { (body_content) }},
	};
	html!(
		(DOCTYPE) html lang = "en" {
			head { (head) }
			(body)
		}
	)
	.into()
}

pub fn user_settings_html(user: User, theme: Theme, sessions: Vec<Session>) -> String {
	let heading = "User Settings";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "User sessions, update, and feedback.";
	let link = format!("{SITE_LINK}/user-settings");
	let user_type = user.user_type.clone();
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		@if user_type == UserType::Admin {
			(update_user_role_html()) hr;
			(ban_user_html()) hr;
		}
		(update_user_html(&user)) hr;
		(user_feedback_html(user)) hr;
		(user_sessions_html(sessions))
	};
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, &link))
		.header(header_html(Some(user_type), Pages::User, &theme))
		.mane(mane)
		.call()
}

fn update_user_html(user: &User) -> PreEscaped<String> {
	let next_fetch_time = user.date_last_fetch + MIN_USER_UPDATE_TIME;
	let button_text = "Update User Info";
	html!(
		h2 { ("Update Info") }
		p { "This site pulls user information from Fimfiction. \
			If you update your name or profile picture on Fimfiction, \
			we have no idea unless you click the button to re-fetch your data." }
		span class = "row" {
			@if let Some(pfp_url) = &user.pfp_url {
				img src = (format!("{pfp_url}-32")) alt = (user.name) {}
				" - "
			}
			(user.name)
		}
		@if Utc::now() > next_fetch_time || user.user_type == UserType::Admin {
			(button_link(button_text, "/user/update"))
		} @else {
			@let remaining = format_milliseconds(
				(next_fetch_time - Utc::now()).num_milliseconds() as u128, Some(2)).unwrap();
			(format!("Please wait {remaining} before trying again."))
			(button_disabled(button_text))
		}
	)
}

pub fn update_user_role_html() -> PreEscaped<String> {
	html! {
		form method = "post" action = "/user/role" {
			h2 { ("Update User Role") }
			p { "Update a user's role based off their Fimfiction ID." }
			span class = "row" {
				label for = "id" { "User ID:" }
				(input_text_numeric_required("id", "id", 1, 8))
			}
			span class = "row" {
				span { "User Role:" }
				input id = "voter" type = "radio" name = "role" value = (UserType::Voter) required {}
				label for = "voter" { (UserType::Voter) }
				input id = "writer" type = "radio" name = "role" value = (UserType::Writer) {}
				label for = "writer" { (UserType::Writer) }
				input id = "admin" type = "radio" name = "role" value = (UserType::Admin) {}
				label for = "admin" { (UserType::Admin) }
			}
			button type = "submit" { "Update User Role" }
		}
	}
}

pub fn ban_user_html() -> PreEscaped<String> {
	html! {
		form method = "post" action = "/user/ban" {
			h2 { ("Ban User") }
			p { "Ban a user based off their Fimfiction ID." }
			span class = "row" {
				label for = "id" { "User ID:" }
				(input_text_numeric_required("id", "id", 1, 8))
			}
			label for = "reason" { "Ban Reason:" }
			(textarea_required("reason", "reason", 8, 256))
			button type = "submit" { "Ban User" }
		}
	}
}

pub fn user_feedback_html(user: User) -> PreEscaped<String> {
	html! {
		form method = "post" action = "/user/feedback" {
			h2 { ("Update User Feedback") }
			label for = "public" { h3  { "Public Feedback" } }
			p { "May appear in a future blog post about this event." }
			(textarea_value("public", "feedback_public", 1_000_000, &user.feedback_public.unwrap_or_default()))
			label for = "private" { h3  { "Private Feedback" } }
			p { "Shared only with the developers and writers of this event." }
			(textarea_value("private", "feedback_private", 1_000_000, &user.feedback_private.unwrap_or_default()))
			button type = "submit" { "Submit Feedback" }
		}
	}
}

pub fn user_sessions_html(sessions: Vec<Session>) -> PreEscaped<String> {
	html! {
		form method = "post" action = "/user/revoke-sessions" {
			h1 { "Sessions" }
			table role = "table" {
				caption role = "caption" { "Active User Sessions" }
				thead role = "rowgroup" {
					tr role = "row" {
						th role = "columnheader" { "Revoke?" }
						th role = "columnheader" { "User Agent" }
						th role = "columnheader" { "Created" }
						th role = "columnheader" { "Last Seen" }
					}
				}
				tbody role = "rowgroup" {
					(session_table_row(&sessions[0], 0))
					@for (num, session) in sessions.iter().enumerate().skip(1) {
						(session_table_row(session, num))
					}
				}
			}
			button type = "submit" { "Revoke Sessions" }
		}
	}
}

fn session_table_row(session: &Session, num: usize) -> PreEscaped<String> {
	html! (
		tr role = "row" {
			td role = "cell" data-cell = "Revoke: "
				{ input type = "checkbox" id = (num) name = (num) value = (session.token) {} }
			@if num == 0 {
				td role = "cell" data-cell = "User Agent: " { b { "(Active) " } (session.user_agent) }
			} @else {
				td role = "cell" data-cell = "User Agent: " { (session.user_agent) }
			}
			td role = "cell" data-cell = "Created: " { (session.date_created.format("%d/%m/%Y %H:%M")) }
			td role = "cell" data-cell = "Last Seen: " { (session.last_seen.format("%d/%m/%Y %H:%M")) }
		}
	)
}

pub fn chapters_html(user: User, theme: Theme, chapters: Vec<ChapterTable>) -> String {
	let heading = "Chapters";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "Chapter list and new chapter page.";
	let link = format!("{SITE_LINK}/chapters");
	let user_type = user.user_type.clone();
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		(chapters_list_html(chapters, user_type == UserType::Admin))
		(new_chapter_html())
	};
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, &link))
		.header(header_html(Some(user_type), Pages::Chapters, &theme))
		.mane(mane)
		.call()
}

pub fn chapters_list_html(chapters: Vec<ChapterTable>, admin: bool) -> PreEscaped<String> {
	html! {
		h2 { "Chapter List" }
		@let mut prev_published: Option<bool> = None;
		@for chapter in chapters.iter() {
			span class = "list-item" {
				(chapter_list_item_html(chapter, &mut prev_published, admin))
			}
		}
	}
}

fn chapter_list_item_html(
	chapter: &ChapterTable, prev_published: &mut Option<bool>, admin: bool,
) -> PreEscaped<String> {
	let active = chapter.meta.fimfic_ch_id.is_some() || chapter.meta.minutes_left.is_some();
	let first_number = !active && chapter.meta.chapter_order.is_some() && prev_published.is_none();
	*prev_published = match first_number {
		true => Some(chapter.meta.fimfic_ch_id.is_some()),
		false => None,
	};
	html! (
		h3 { a href = (format!("/chapters/{}", chapter.meta.id)) { (chapter.last_data.title) sup { "↗" } } }
		p {
			b { " Ch Order: " }
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
			b { " Vote Duration: " }
			@if !active && admin {
				@let endpoint = format!("/chapters/{}/vote-duration/1", chapter.meta.id);
				(button_link("▲", &endpoint))
			}
			(chapter.meta.vote_duration)
			@if !active && admin {
				@let endpoint = format!("/chapters/{}/vote-duration/-1", chapter.meta.id);
				(button_link("▼", &endpoint))
			}
			@if let Some(minutes_left) = chapter.meta.minutes_left {
				b { " Min Left: " }
				@if admin && minutes_left != 0 {
					@let endpoint = format!("/chapters/{}/minutes-left/1", chapter.meta.id);
					(button_link("▲", &endpoint))
				}
				(minutes_left)
				@if admin && minutes_left != 0 {
					@let endpoint = format!("/chapters/{}/minutes-left/-1", chapter.meta.id);
					(button_link("▼", &endpoint))
				}
			}
		}
		p {
			b { "Questions: " }
			a href = (format!("/chapters/{}/questions", chapter.meta.id)) { (chapter.questions) sup { "↗" } }
			b { " Revisions: " }
			a href = (format!("/chapters/{}/revisions", chapter.meta.id)) { (chapter.revisions) sup { "↗" } }
			@if let Some(id) = chapter.meta.fimfic_ch_id {
				b { " Fimfic Ch ID: " }
				a href = (format!("https://www.fimfiction.net/chapter/{id}")) { (id) sup { "↗" } }
			}
			b { " Intro/Outro Word Count: " }
			(chapter.last_data.intro_text.clone().map(|text| count_words(&text)).unwrap_or_default())
			"/"
			(chapter.last_data.outro_text.clone().map(|text| count_words(&text)).unwrap_or_default())
		}
		p {
			b { "Last Edit: " }
			(chapter.meta.last_edit.format("%y-%m-%d %H:%M"))
			b { " Last Revision: " }
			(chapter.last_data.date_created.format("%y-%m-%d %H:%M"))
			b { " Created: " }
			(chapter.first_data.date_created.format("%y-%m-%d %H:%M"))
		}
	)
}

pub fn new_chapter_html() -> PreEscaped<String> {
	html! {
		form method = "post" action = "/chapters" {
			h2 { "New Chapter" }
			@let name = "title";
			label for = (name) { "Title:" }
			(input_text_required(name, name, 8, 256))
			@let name = "intro_text";
			label for = (name) { "Intro:" }
			(textarea(name, name, 1_000_000))
			@let name = "outro_text";
			label for = (name) { "Outro:" }
			(textarea(name, name, 1_000_000))
			button type = "submit" { "Create Chapter" }
		}
	}
}

pub fn edit_chapter_html(
	user: User, theme: Theme, chapter: Chapter, data: ChapterRevision,
) -> String {
	let heading = "Edit Chapter";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "Make changes to the specified chapter.";
	let link = format!("{SITE_LINK}/chapters/{}", chapter.id);
	let user_type = user.user_type.clone();
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		form method = "post" action = (format!("/chapters/{}", chapter.id)) {
			@let name = "title";
			label for = (name) { "Title:" }
			(input_text_value_required(name, name, 8, 256, &data.title))
			@let name = "intro_text";
			label for = (name) { "Intro:" }
			(textarea_value(name, name, 1_000_000, &data.intro_text.unwrap_or_default()))
			@let name = "outro_text";
			label for = (name) { "Outro:" }
			(textarea_value(name, name, 1_000_000, &data.outro_text.unwrap_or_default()))
			button type = "submit" { "Save Chapter" }
		}
	};
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, &link))
		.header(header_html(Some(user_type), Pages::Chapters, &theme))
		.mane(mane)
		.call()
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
						option value = (QuestionType::MultipleChoice) { (QuestionType::MultipleChoice) }
						option value = (QuestionType::Multiselect) { (QuestionType::Multiselect) }
						option value = (QuestionType::Scale) { (QuestionType::Scale) }
					}
					br;
					@let name = "response_percent";
					label for = (name) { "Response Percentage:" }
					br;
					(input_text_float_required(name, name))
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
					h1 { "Edit Question" }
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
					(input_text_float_value_required(name, name, data.response_percent))
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
				option value = (QuestionType::MultipleChoice) selected { (QuestionType::MultipleChoice) }
				option value = (QuestionType::Multiselect) { (QuestionType::Multiselect) }
				option value = (QuestionType::Scale) { (QuestionType::Scale) }
			 },
			 QuestionType::Multiselect => {
				option value = (QuestionType::MultipleChoice) { (QuestionType::MultipleChoice) }
				option value = (QuestionType::Multiselect) selected { (QuestionType::Multiselect) }
				option value = (QuestionType::Scale) { (QuestionType::Scale) }
			 },
			 QuestionType::Scale => {
				option value = (QuestionType::MultipleChoice) { (QuestionType::MultipleChoice) }
				option value = (QuestionType::Multiselect) { (QuestionType::Multiselect) }
				option value = (QuestionType::Scale) selected { (QuestionType::Scale) }
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
						th { "Response Percent" br; "/Ponies" } // done
						th { "Chapter" br; "Order" } // done
						th { "Options" } // done?
						th { "Outcomes" } // done?
						th { "Revisions" } // done
						th { "Claiment" } // done
						th { "Last" br; "Edit" } // done
						th { "Last" br; "Revision" } // done
						th { "Created" } // done
						th { "Edit" } // done
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
				(question.last_data.response_percent) "%" br;
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

pub fn questions_html(questions: Vec<QuestionTable>, population: u32, user: User) -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				h1 { "Questions" }
				br;
				table {
					tr {
						th { "ID" } // done
						th { "Chapter" br; "Number" } // done
						th { "Chapter" br; "Order" } // done
						th { "Question" } // done
						th { "Question" br; "Type" } // done
						th { "Response Percent" br; "/Ponies" } // done
						th { "Options" } // done?
						th { "Outcomes" } // done?
						th { "Revisions" } // done
						th { "Claiment" } // done
						th { "Last" br; "Edit" } // done
						th { "Last" br; "Revision" } // done
						th { "Created" } // done
						th { "Edit" } // done
					}
					@for question in questions.into_iter() {
						(questions_table(question, population, &user))
					}
				}
				(button_link("New Question", "/questions/new"))
			};
		};
	}
	.into()
}

pub fn questions_table(
	question: QuestionTable, population: u32, user: &User,
) -> PreEscaped<String> {
	html! {
		tr {
			td { (question.meta.id) }
			td { (question.meta.chapter_id.map(|id| id.to_string()).unwrap_or_default()) }
			td { (question.meta.chapter_order.map(|order| order.to_string()).unwrap_or_default()) }
			td { (question.last_data.question_text) }
			td { (question.last_data.question_type) }
			td {
				(question.last_data.response_percent) "%" br;
				(format_number_u128((population as f64 * question.last_data.response_percent / 100.0).round() as u128).unwrap())
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

pub fn head_html(title: &str, description: &str, link: &str) -> PreEscaped<String> {
	html! {
		title { (title) };
		meta charset = "UTF-8";
		meta http-equiv = "X-UA-Compatible" content = "IE=edge";
		meta name = "viewport" content = "width=device-width,initial-scale=1";
		link rel = "stylesheet" crossorigin href = "/style.css";
		meta name = "theme-color" content = { "#F5B7D0" };
		link rel = "canonical" href = (link);
		meta property = "og:title" content = (title);
		meta property = "og:description" content = (description);
		meta property = "og:url" content = (link);
		meta property = "og:type" content = "website";
		meta property = "og:site_name" content = (SITE_NAME);
		@let encode = encode_url(title);
		link
			crossorigin
			rel = "alternate"
			type = "application/json+oembed"
			href = { "https://www.fixfiction.net/oembed?" (encode) }
			title = (title);
		script crossorigin src = "/mane.js" {}

	}
}

fn encode_url(title: &str) -> String {
	let mut encode = form_urlencoded::Serializer::new(String::new());
	encode.append_pair("type", "rich");
	encode.append_pair("version", "1.0");
	encode.append_pair("provider_name", SITE_NAME);
	encode.append_pair("provider_url", SITE_LINK);
	encode.append_pair("title", title);
	encode.append_pair("cache_age", "86400");
	encode.append_pair("html", "");
	encode.finish()
}

fn header_html(user_type: Option<UserType>, page: Pages, theme: &Theme) -> PreEscaped<String> {
	html!(
		fieldset class = "logo" {
			input id = "census" type = "radio" name = "logo" onclick = "submitLogo('census')" {}
			label for = "census" { "Census" }
			input id = "consensus" type = "radio" name = "logo" onclick = "submitLogo('consensus')" {}
			label for = "consensus" { "Consensus" }
		}
		nav {
			span class = "nav" {
				(header_link_html("/", "Home", page == Pages::Home))
				@if user_type.is_some() {
					(header_link_html("/user", "User", page == Pages::User))
				}
				(header_link_html("/about", "About", page == Pages::About))
			}
			@if let Some(user_type) = user_type && user_type != UserType::Voter {
				span class = "nav" {
					(header_link_html("/chapters", "Chapters", page == Pages::Chapters))
					(header_link_html("/questions", "Questions", page == Pages::Questions))
					(header_link_html("/feedback", "Feedback", page == Pages::Feedback))
				}
			}
		}
		fieldset class = "theme" {
			span { "Theme:" }
			(header_theme_html("light", "Celestia", theme == &Theme::Light))
			(header_theme_html("dark", "Luna", theme == &Theme::Dark))
		}
	)
}

fn header_link_html(link: &str, text: &str, checked: bool) -> PreEscaped<String> {
	html!(
		@if checked {
			input type = "radio" name = "page" checked {}
			a href = (link) { (text) }
		} @ else {
			input type = "radio" name = "page" disabled {}
			a href = (link) { (text) }
		}
	)
}

fn header_theme_html(theme: &str, text: &str, checked: bool) -> PreEscaped<String> {
	let on_click = format!("updateTheme('{theme}')");
	html!(
		@if checked {
			input id = (theme) type = "radio" name = "theme" onchange = (on_click) checked {}
			label for = (theme) { (text) }
		} @ else {
			input id = (theme) type = "radio" name = "theme" onchange = (on_click) {}
			label for = (theme) { (text) }
		}
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

fn input_text_float_required(id: &str, name: &str) -> PreEscaped<String> {
	html!	(
		input
			id = (id)
			type = "text"
			name = (name)
			minlength = "1"
			maxlength = "12"
			pattern = r"[0-9]*[.]?[0-9]*"
			required {}
	)
}

fn input_text_float_value_required(id: &str, name: &str, value: f64) -> PreEscaped<String> {
	html!	(
		input
			id = (id)
			type = "text"
			name = (name)
			minlength = "1"
			maxlength = "12"
			pattern = r"[0-9]*[.]?[0-9]*"
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
