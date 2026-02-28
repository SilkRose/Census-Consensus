use std::fs;

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
		.head(head_html(&title, description, &link, &theme))
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
			(user_inline_html(user))
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
			(markdown_preamble())
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
			td role = "cell" data-cell = "Created: " { (session.date_created.format("%y-%m-%d %H:%M")) }
			td role = "cell" data-cell = "Last Seen: " { (session.last_seen.format("%y-%m-%d %H:%M")) }
		}
	)
}

pub fn chapters_html(user: User, theme: Theme, chapters: Vec<ChapterTable>) -> String {
	let heading = "Chapters";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "Chapter list and new chapter page.";
	let link = format!("{SITE_LINK}/chapters");
	let user_type = user.user_type.clone();
	let admin = user_type == UserType::Admin;
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		h2 { "Chapter List" }
		@let mut prev_published: Option<bool> = None;
		@for chapter in chapters.iter() {
			span class = "list-item" {
				(chapter_list_item_html(chapter, &mut prev_published, admin))
			}
		}
		(new_chapter_html())
	};
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, &link, &theme))
		.header(header_html(Some(user_type), Pages::Chapters, &theme))
		.mane(mane)
		.call()
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
		h3 { a href = (format!("/chapters/{}", chapter.meta.id)) { (chapter.newest_data.title) sup { "↗" } } }
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
			(chapter.newest_data.intro_text.clone().map(|text| count_words(&text)).unwrap_or_default())
			"/"
			(chapter.newest_data.outro_text.clone().map(|text| count_words(&text)).unwrap_or_default())
		}
		p {
			b { "Last Edit: " }
			(chapter.meta.last_edit.format("%y-%m-%d %H:%M"))
			b { " Last Revision: " }
			(chapter.newest_data.date_created.format("%y-%m-%d %H:%M"))
			b { " Created: " }
			(chapter.oldest_data.date_created.format("%y-%m-%d %H:%M"))
		}
	)
}

pub fn new_chapter_html() -> PreEscaped<String> {
	html! {
		form method = "post" action = "/chapters" {
			h2 { "New Chapter" }
			@let name = "title";
			label for = (name) { "Title:" }
			(input_text_required(name, name, 1, 256))
			(markdown_preamble())
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
			(input_text_value_required(name, name, 1, 256, &data.title))
			(markdown_preamble())
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
		.head(head_html(&title, description, &link, &theme))
		.header(header_html(Some(user_type), Pages::Chapters, &theme))
		.mane(mane)
		.call()
}

pub fn chapter_history_html(user: User, theme: Theme, chapter: ChapterData) -> String {
	let heading = "Chapter Revisions";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "Chapter version history.";
	let link = format!("{SITE_LINK}/chapters/{}/revisions", chapter.meta.id);
	let user_type = user.user_type.clone();
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		@for revision in chapter.data.into_iter() {
			details class = "list-item" name = "revision" {
				summary {
					"Date: " (revision.date_created.format("%y-%m-%d %H:%M"))
					" By: "
					@let user = chapter.users.get(&revision.id).expect("User will always be present.");
					(user_inline_html(user))
				}
				h3 { "title:" }
				(revision.title)
				h3 { "Intro:" }
				pre class = "left-text" {
					(revision.intro_text.unwrap_or_default())
				}
				h3 { "outro:" }
				pre class = "left-text" {
					(revision.outro_text.unwrap_or_default())
				}
			}
		}
	};
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, &link, &theme))
		.header(header_html(Some(user_type), Pages::Chapters, &theme))
		.mane(mane)
		.call()
}

pub fn feedback_html(user: User, theme: Theme, users: Vec<User>) -> String {
	let heading = "User Feedback";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "Public and private feedback from every user.";
	let link = format!("{SITE_LINK}/feedback");
	let user_type = user.user_type.clone();
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		@for user in users {
			span class = "list-item" {
				h2 { (user.name) }
				@if let Some(pfp_url) = &user.pfp_url {
					img src = (format!("{pfp_url}-64")) alt = (user.name) {}
				}
				@if let Some(public) = user.feedback_public {
					h3 { "Public:" }
					pre class = "left-text" {
						(public)
					}
				}
				@if let Some(private) = user.feedback_private {
					h3 { "Private:" }
					pre class = "left-text" {
						(private)
					}
				}
			}
		}
	};
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, &link, &theme))
		.header(header_html(Some(user_type), Pages::Feedback, &theme))
		.mane(mane)
		.call()
}

pub fn questions_html(
	user: User, theme: Theme, questions: Vec<QuestionTable>, population: u32,
) -> String {
	let heading = "Questions";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "Question list and new question page.";
	let link = format!("{SITE_LINK}/questions");
	let user_type = user.user_type.clone();
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		h2 { "Question List" }
		@for question in questions.into_iter() {
			span class = "list-item" {
				(question_list_item_html(question, population, &user, None))
			}
		}
		(new_question_html(None))
	};
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, &link, &theme))
		.header(header_html(Some(user_type), Pages::Questions, &theme))
		.mane(mane)
		.call()
}

pub fn chapter_questions_html(
	user: User, theme: Theme, chapter: Chapter, questions: Vec<QuestionTable>, population: u32,
) -> String {
	let heading = "Chapter Questions";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "Question list and new question page for the selected chapter.";
	let link = format!("{SITE_LINK}/questions");
	let user_type = user.user_type.clone();
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		h2 { "Question List" }
		@for question in questions.into_iter() {
			span class = "list-item" {
				(question_list_item_html(question, population, &user, Some(&chapter)))
			}
		}
		(new_question_html(Some(&chapter)))
	};
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, &link, &theme))
		.header(header_html(Some(user_type), Pages::Questions, &theme))
		.mane(mane)
		.call()
}

pub fn question_list_item_html(
	question: QuestionTable, population: u32, user: &User, chapter: Option<&Chapter>,
) -> PreEscaped<String> {
	html! {
		h3 { a href = (format!("/questions/{}", question.meta.id)) { (question.newest_data.question_text) sup { "↗" } } }
		p {
			b { "Asked By: " }
			(question.newest_data.asked_by)
		}
		p {
			@if let Some(chapter) = chapter {
				b { "Ch Order: " }
				@if let Some(order) = question.meta.chapter_order {
					@if order > 1 {
						@let endpoint = format!("/chapters/{}/questions/{}/ordered/-1", chapter.id, question.meta.id);
						(button_link("▲", &endpoint))
					} @else {
						(button_disabled("▲"))
					}
					(order)
					@let endpoint = format!("/chapters/{}/questions/{}/ordered/1", chapter.id, question.meta.id);
					(button_link("▼", &endpoint))
				} @else {
					@let endpoint = format!("/chapters/{}/questions/{}/ordered", chapter.id, question.meta.id);
					(button_link("Add", &endpoint))
				}
			} @else if let Some(chapter_id) = question.meta.chapter_id {
				@ if let Some(chapter_order) = question.meta.chapter_order {
					b { "Ch ID/Order: " }
					(chapter_id) "/" (chapter_order)
				} @ else {
					b { "Ch ID: " }
					(chapter_id)
				}
			}
			b { " Type: " }
			(question.newest_data.question_type)
			b { " Res %/Ponies: " }
			(question.newest_data.response_percent) "%/"
			(format_number_u128((population as f64 * question.newest_data.response_percent / 100.0).round() as u128).unwrap())
		}
		p {
			b { "Options/Results: " }
			(question.options)
			"/"
			(question.outcomes)
			b { " Revisions: " }
			a href = (format!("/questions/{}/revisions", question.meta.id)) { (question.revisions) sup { "↗" } }
			@if let Some(claiment) = question.claiment {
				b { " Claiment: " }
				(user_inline_html(&claiment))
				@if user.id == claiment.id {
					@let endpoint = format!("/questions/{}/unclaim", question.meta.id);
					(button_link("Un-Claim", &endpoint))
				} @else if user.user_type == UserType::Admin {
					@let endpoint = format!("/questions/{}/unclaim", question.meta.id);
					(button_link("Revoke", &endpoint))
				}
			} @ else {
				@let endpoint = format!("/questions/{}/claim", question.meta.id);
				(button_link("Claim", &endpoint))
			}
		}
		p {
			b { "Last Edit: " }
			(question.meta.last_edit.format("%y-%m-%d %H:%M"))
			b { " Last Revision: " }
			(question.newest_data.date_created.format("%y-%m-%d %H:%M"))
			b { " Created: " }
			(question.oldest_data.date_created.format("%y-%m-%d %H:%M"))
		}
	}
}

pub fn new_question_html(chapter: Option<&Chapter>) -> PreEscaped<String> {
	let link = match chapter {
		Some(chapter) => &format!("/questions?chapter_id={}", chapter.id),
		None => "/questions",
	};
	html! {
		form method = "post" action = (link) {
			h2 { "New Question" }
			@if chapter.is_some() {
				p { "Questions created on a chapter's question page "
				"are automatically assigned to that chapter." }
			}
			@let name = "question_text";
			h3 { label for = (name) { "Question:" } }
			(input_text_required(name, name, 1, 256))
			@let name = "question_type";
			h3 { label for = (name) { "Question Type: " } }
			select name = (name) id = (name) {
				option value = (QuestionType::MultipleChoice) { (QuestionType::MultipleChoice) }
				option value = (QuestionType::Multiselect) { (QuestionType::Multiselect) }
				option value = (QuestionType::Scale) { (QuestionType::Scale) }
			}
			h3 { "Claim:" }
			span class = "row" {
				@let name = "claimed";
				input type = "checkbox" id = (name) name = (name) value = "true" {}
				label for = (name) { "Claim on creation." }
			}
			@let name = "response_percent";
			h3 { label for = (name) { "Response Percentage:" } }
			(input_text_float_required(name, name))
			@let name = "asked_by";
			h3 { label for = (name) { "Asked by:" } }
			(input_text_required(name, name, 1, 256))
			@let name = "option_writing";
			h3 { label for = (name) { "Options:" } }
			(option_explanation())
			(textarea(name, name, 1_000_000))
			@let name = "result_writing";
			h3 { label for = (name) { "Result Writings:" } }
			(result_explanation())
			(markdown_preamble())
			(textarea(name, name, 1_000_000))
			button type = "submit" { "Create Question" }
		}
	}
}

pub fn edit_question_html(
	user: User, theme: Theme, question: Question, data: QuestionRevision, population: u32,
) -> String {
	let heading = "Edit Question";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "Edit a question.";
	let link = format!("{SITE_LINK}/questions/{}", question.id);
	let user_type = user.user_type.clone();
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		form method = "post" action = (format!("/questions/{}", question.id)) {
			@let name = "question_text";
			h3 { label for = (name) { "Question:" } }
			(input_text_value_required(name, name, 1, 256, &data.question_text))
			@let name = "question_type";
			h3 { label for = (name) { "Question Type:" } }
			select name = (name) id = (name) {
				(question_type_match(data.question_type))
			}
			@let name = "response_percent";
			h3 { label for = (name) { "Response Percentage:" } }
			(input_text_float_value_required(name, name, data.response_percent))
			"Ponies answered: "
				(format_number_u128((population as f64 * data.response_percent / 100.0).round() as u128).unwrap())
				" out of: "
				(format_number_u128(population as u128).unwrap())
				" -- Updated on refresh."
			@let name = "asked_by";
			h3 { label for = (name) { "Asked by:" } }
			(input_text_value_required(name, name, 1, 256, &data.asked_by))
			@let name = "option_writing";
			h3 { label for = (name) { "Options:" } }
			(option_explanation())
			(textarea_value(name, name, 1_000_000, &data.option_writing.unwrap_or_default()))
			@let name = "result_writing";
			h3 { label for = (name) { "Result Writings:" } }
			(result_explanation())
			(markdown_preamble())
			(textarea_value(name, name, 1_000_000, &data.result_writing.unwrap_or_default()))
			button type = "submit" { "Save Question" }
		}
	};
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, &link, &theme))
		.header(header_html(Some(user_type), Pages::Questions, &theme))
		.mane(mane)
		.call()
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

pub fn question_history_html(
	user: User, theme: Theme, question: QuestionData, population: u32,
) -> String {
	let heading = "Question Revision History";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "All revisions for a given question.";
	let link = format!("{SITE_LINK}/questions/{}/revisions", question.meta.id);
	let user_type = user.user_type.clone();
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		@for revision in question.data.into_iter() {
			details class = "list-item" name = "revision" {
				summary {
					"Date: " (revision.date_created.format("%y-%m-%d %H:%M"))
					" By: "
					@let user = question.users.get(&revision.id).expect("User will always be present.");
					(user_inline_html(user))
				}
				h3 { "Question:" }
				(revision.question_text)
				h3 { "Question Type:" }
				(revision.question_type)
				h3 { "Response percent:" }
				(revision.response_percent)
				h3 { "Ponies answered:" }
				(format_number_u128((population as f64 * revision.response_percent / 100.0).round() as u128).unwrap())
				" out of: "
				(format_number_u128(population as u128).unwrap())
				h3 { "Asked By:" }
				(revision.asked_by)
				@if let Some(opt) = revision.option_writing {
					h3 { "Options:" }
					pre class = "left-text" {
						(opt)
					}
				}
				@if let Some(res) = revision.result_writing {
					h3 { "Results:" }
					pre class = "left-text" {
						(res)
					}
				}
			}
		}
	};
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, &link, &theme))
		.header(header_html(Some(user_type), Pages::Questions, &theme))
		.mane(mane)
		.call()
}

// HTML components go below this comment:

pub fn head_html(title: &str, description: &str, link: &str, theme: &Theme) -> PreEscaped<String> {
	html! {
		title { (title) };
		meta charset = "UTF-8";
		meta http-equiv = "X-UA-Compatible" content = "IE=edge";
		meta name = "viewport" content = "width=device-width,initial-scale=1";
		link rel = "stylesheet" crossorigin href = "/style.css";
		(theme_color_html(theme))
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

fn theme_color_html(theme: &Theme) -> PreEscaped<String> {
	html! {
		@match theme {
			 Theme::Light => meta name = "theme-color" content = { "#f2d9e8" };
			 Theme::Dark => meta name = "theme-color" content = { "#aba4f4" };
			 Theme::None => {
				meta name = "theme-color" content = { "#f2d9e8" };
				meta name = "theme-color" content = { "#f2d9e8" } media = "(prefers-color-scheme: light)";
				meta name = "theme-color" content = { "#aba4f4" } media = "(prefers-color-scheme: dark)";
			 },
		}
	}
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

fn user_inline_html(user: &User) -> PreEscaped<String> {
	html! {
		@if let Some(pfp_url) = &user.pfp_url {
			img src = (format!("{pfp_url}-32")) alt = (user.name) {}
			" - "
		}
		(user.name)
	}
}

fn markdown_preamble() -> PreEscaped<String> {
	html! {
		p {
			"Please use "
			a href = "https://www.markdownguide.org/cheat-sheet/" { "Markdown" sup { "↗" } }
			" for any and all text formatting."
		}
	}
}

fn option_explanation() -> PreEscaped<String> {
	html! {
		p { "All option types support comment lines." br;
		"To make a comment start the line with: " b { "//" } "." }
		h4 { "Scale Option Formatting" }
		p {
			"A scale question would be like this:" br;
			"'On a scale from 1 to 10, how much do you love Pinkie Pie?'" br;
			"Scale options consist of two lines:"
		}
		ol class = "left-text" {
			li { b { "[1-10]" } ": The first number is the start of the scale, and the second the end of the scale." }
			li { b { "Order: 5, 4, 2, 3, 1, 7, 6, 8, 10, 9" } ": An ordering of the options for priority to prevent ties." }
		}
		p { "An example of a scale option would be:" }
		span class = "left-text" {
			"// the scale options:" br;
			"[1-5]" br;
			"// the order in which to break ties:" br;
			"Order: 3, 2, 1, 5, 4"
		}
		h4 { "Multiple Choice/Multi-Select Option Formatting" }
		p {
			"These question types share the same option formatting." br;
			"The only difference is that Multiple Choice can only have one answer selected," br;
			"while Multi-Select questions can have multiple answers checked." br;
			"These question types have two option line types:"
		}
		ol class = "left-text" {
			li { b { "A: [option text]" } ": The A is the option ID, a colon and a space, then the text of the option." }
			li { b { "Order: A, C, B, D" } " An ordering of the options for priority to prevent ties." }
		}
		p { "An example of these options would be:" }
		span class = "left-text" {
			"// The first option:" br;
			"A: Pinkie Pie" br;
			"// The second option:" br;
			"B: Twilight Sparkle" br;
			"// The third option:" br;
			"C: Fluttershy" br;
			"// the order in which to break ties:" br;
			"Order: A, C, B"
		}
	}
}

fn result_explanation() -> PreEscaped<String> {
	html! {
		p { "Result writings support comment lines." br;
		"To make a comment start the line with: " b { "//" } "." }
		h4 { "Result Writing Formatting" }
		p {
			"Result writings are the text that gets inserted into the chapter when published." br;
			"A result writing starts with a # followed by the condition for the vote answers." br;
			"Here are examples with explanations:"
		}
		ol class = "left-text" {
			li { b { "# A > 50%" } ": The option with id A won more than 50% of the votes." }
			li { b { "# B > 40%, C > 30%" } ": Option B got over 30% and C got over 30%." }
			li { b { "# A > 1/3" } ": Option A won with over 1/3 of all votes." }
			li { b { "# A > B" } ": Option A won with more votes than B." }
			li { b { "# A" } ": Option A won with the most votes." }
		}
		p {
			"As you can see above, result writtings support both fractions and percentages." br;
			"Multiple condictions can be used, the first option that matches is the one that gets posted." br;
			"An example of result writings is as such:"
		}
		span class = "left-text" {
			"// if option A is over 30% this result will be put into the chapter." br;
			"# A > 30%" br;
			"Oh, wow! Twilight, I can't beleive you are so cute!" br;
			"// if A has more votes than B, but didn't pass the first writing condition, this will get posted." br;
			"# A > B" br;
			"Oh, wow! Twilight and Pinkie are so cute!"
		}
		p {
			"Now, while writing your results, you might want "
			"to directly quote the number or percentage of the votes or winning option."
			" This is supported with a set of replacement strings explained below:"
		}
		p {
			"Replacements use identifiers to work, the following is a list of all identifiers:"
		}
		ol class = "left-text" {
			li { b { "vp" } ": The vote percent." }
			li { b { "vcc" } ": The count of votes. Ex: 1,234,567" }
			li { b { "vcw" } ": The count using the biggest word. Ex 10 million" }
			li { b { "p-" } ": Prepended to an identified for result placements where it is unknown." }
			li { b { "name" } ": The text of an option." }
			li { b { "question" } ": The text of the question." }
		}
		p {
			"These must be used with the following symbols that get replaced by you when writing:"
		}
		ol class = "left-text" {
			li { b { "id" } ": The id for a known option." }
			li { b { ".d" } ": The number of decimal places to show for numbers/percentages." }
			li { b { "x" } ": The position for an unknown result." }
		}
		p {
			"Here are some examples of how to use them:" br;
			"(Each item explains what's new/changes from the previous one.)"
		}
		ol class = "left-text" {
			li { b { "%A[vp]%" } ": A is the option id, vp is vote percent. ex 23%" }
			li { b { "%A[vp.2]%" } ": .2 is the decimal places. ex 23.23%" }
			li { b { "%B[vcc]%" } ": B is the option id, vcc is the vote count. Ex: 1,234,567" }
			li { b { "%C[vcw]%" } ": C is the option, vcw is count in words. Ex 10 million" }
			li { b { "%C[vcw.1]%" } ": .1 is the decimal places. Ex 10.1 million" }
			li { b { "%3[p-name]%" } ": 3 is the placement, p- is placement prepend, name is the text." }
			li { b { "%2[p-vp]%" } ": 2 is the placement, vp is vote percent. Ex 56%" }
			li { b { "%2[p-vp.2]%" } ": .2 is 2 decimal places. Ex 56.43%" }
			li { b { "%4[p-vcc]%" } ": 4 is the placement and vcc is the vote count. Ex 21,657,541" }
			li { b { "%2[p-vcw]%" } ": 2 is the placement and vcw is count in words. Ex 40 million" }
			li { b { "%2[p-vcw.1]%" } ": .1 is the decimal places. Ex 40.1 million" }
			li { b { "%[question]%" } ": The text of the question." }
		}
		p { "Here is a complete example:" }
		span class = "left-text" {
			(fs::read_to_string("./assets/writing-result-example.md").unwrap())
		}
	}
}
