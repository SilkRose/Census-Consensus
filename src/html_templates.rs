use crate::endpoints::MIN_USER_UPDATE_TIME;
use crate::structs::*;
use crate::theme::Theme;
use crate::utility::{construct_question_data, count_words, parse_options};
use crate::{SITE_LINK, SITE_NAME, result_formatter};
use bon::builder;
use chrono::Utc;
use maud::{DOCTYPE, PreEscaped, html};
use pony::markdown::WarningType;
use pony::markdown::html::parse;
use pony::number_format::format_number_u128;
use pony::smart_map::SmartMap;
use pony::time::format_milliseconds;
use std::collections::HashMap;
use std::fs;
use url::form_urlencoded;

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
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		h2 { "Chapter List" }
		@let mut prev_published = false;
		@for chapter in chapters.iter() {
			span class = "list-item" {
				(chapter_list_item_html(chapter, &mut prev_published, &user))
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
	chapter: &ChapterTable, prev_published: &mut bool, user: &User,
) -> PreEscaped<String> {
	let active = chapter.meta.fimfic_ch_id.is_some() || chapter.meta.minutes_left.is_some();
	let first_number = !active && chapter.meta.chapter_order.is_some() && !*prev_published;
	*prev_published = *prev_published || first_number;
	html! (
		h3 { a href = (format!("/chapters/{}", chapter.meta.id)) { (chapter.newest_data.title) sup { "↗" } } }
		@if chapter.questions > 0 && chapter.meta.fimfic_ch_id.is_some() {
			p {
				a href = (format!("/chapters/{}/survey", chapter.meta.id)) { b { "Survey" } sup { "↗" } } " "
				a href = (format!("/chapters/{}/event-results", chapter.meta.id)) { b { "Event Results" } sup { "↗" } } " "
				a href = (format!("/chapters/{}/live-results", chapter.meta.id)) { b { "Live Results" } sup { "↗" } } " "
				a href = (format!("/chapters/{}/random-results", chapter.meta.id)) { b { "Random Results" } sup { "↗" } }
			}
		}
		p {
			b { "Ch Order: " }
			@if let Some(order) = chapter.meta.chapter_order {
				(order)
			}
			b { " Vote Duration: " }
			(chapter.meta.vote_duration)
			@if let Some(minutes_left) = chapter.meta.minutes_left {
				b { " Min Left: " }
				(minutes_left)
			}
			@if chapter.meta.fimfic_ch_id.is_some() {
				" "
				@if user.user_type != UserType::Voter {
					a href = (format!("/chapters/{}/update", chapter.meta.id)) { b { "Update fimfic Chapter" } sup { "↗" } }
				}
			}
		}
		p {
			@if chapter.questions > 0 {
				b { "Questions: " }
				a href = (format!("/chapters/{}/questions", chapter.meta.id)) { (chapter.questions) sup { "↗" } }
			}
			b { " Revisions: " }
			a href = (format!("/chapters/{}/revisions", chapter.meta.id)) { (chapter.revisions) sup { "↗" } }
			@if let Some(id) = chapter.meta.fimfic_ch_id {
				b { " Fimfic Ch ID: " }
				a href = (format!("https://www.fimfiction.net/chapter/{id}")) { (id) sup { "↗" } }
			}
			b { " Intro/Outro Word Count: " }
			(format_number_u128(
				chapter.newest_data.intro_text.clone().map(|text| count_words(&text)).unwrap_or_default()
				as u128).unwrap())
			"/"
			(format_number_u128(
				chapter.newest_data.outro_text.clone().map(|text| count_words(&text)).unwrap_or_default()
				as u128).unwrap())
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
			button type = "submit" disabled { "Create Chapter" }
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
			@if user.user_type == UserType::Voter {
				button type = "submit" disabled { "Save Chapter" }
			} @else {
				button type = "submit" { "Save Chapter" }
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

pub fn feedback_html(user: User, theme: Theme, users: Vec<UserData>) -> String {
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
				h2 { a href = (format!("https://www.fimfiction.net/user/{}/", user.meta.id)) { (user.meta.name) sup { "↗" } } }
				br;
				@if let Some(pfp_url) = &user.meta.pfp_url {
					img src = (format!("{pfp_url}-64")) alt = (user.meta.name) {}
				}
				h3 { "Logo Stats:" }
				p {
					b { "Census Clicks: " }
					(format_number_u128(user.logo_census as u128).unwrap())
					b { " Consensus Clicks: " }
					(format_number_u128(user.logo_consensus as u128).unwrap())
					b { " Total Clicks: " }
					(format_number_u128((user.logo_census + user.logo_consensus) as u128).unwrap())
				}
				@if let Some(public) = user.meta.feedback_public {
					h3 { "Public:" }
					pre class = "left-text" {
						(public)
					}
				}
				@if let Some(private) = user.meta.feedback_private {
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
	user: User, theme: Theme, questions: Vec<QuestionTable>,
	chapters: SmartMap<i32, (Chapter, ChapterRevision)>, population: i32,
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
			@let chapter = QuestionChapter::Questions(
				question.meta.chapter_id
					.and_then(|id| chapters.get(&id))
					.map(|c| (**c).clone())
			);
			span class = "list-item" {
				(question_list_item_html(question, population, chapter))
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
	user: User, theme: Theme, chapter: Chapter, questions: Vec<QuestionTable>, population: i32,
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
				(question_list_item_html(question, population, QuestionChapter::ChapterQuestions))
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
	question: QuestionTable, population: i32, chapter: QuestionChapter,
) -> PreEscaped<String> {
	html! {
		h3 { a href = (format!("/questions/{}", question.meta.id)) { (question.newest_data.question_text) sup { "↗" } } }
		p {
			b { "Asked By: " }
			(question.newest_data.asked_by)
			@if let QuestionChapter::Questions(ref chapter) = chapter
			&& let Some((chapter, revision)) = chapter {
				b { " Chapter: " }
				a href = (format!("/chapters/{}", chapter.id)) { (revision.title) sup { "↗" } }
			}
		}
		p {
			a href = (format!("/questions/{}/preview", question.meta.id)) { b { "Preview" } sup { "↗" } }
			@if let QuestionChapter::ChapterQuestions = chapter {
				b { " Ch Order: " }
				@if let Some(order) = question.meta.chapter_order {
					(order)
				}
			} @else if let QuestionChapter::Questions(ref chapter) = chapter {
				@if let Some((chapter, _)) = chapter
					&& let Some(chapter_order) = chapter.chapter_order
					&& let Some(question_order) = question.meta.chapter_order {
					b { " Ch Num/Order: " }
					(chapter_order) "/" (question_order)
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
			@if let Some(claimant) = question.claimant {
				b { " Claimant: " }
				(user_inline_html(&claimant))
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
			(options_explanation())
			(textarea(name, name, 1_000_000))
			@let name = "result_writing";
			h3 { label for = (name) { "Result Writings:" } }
			(results_explanation())
			(markdown_preamble())
			(textarea(name, name, 1_000_000))
			button type = "submit" disabled { "Create Question" }
		}
	}
}

pub fn edit_question_html(
	user: User, theme: Theme, question: Question, data: QuestionRevision, population: i32,
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
			(options_explanation())
			(textarea_value(name, name, 1_000_000, &data.option_writing.unwrap_or_default()))
			@let name = "result_writing";
			h3 { label for = (name) { "Result Writings:" } }
			(results_explanation())
			(markdown_preamble())
			(textarea_value(name, name, 1_000_000, &data.result_writing.unwrap_or_default()))
			@if user.user_type == UserType::Voter {
				button type = "submit" disabled { "Save Question" }
			} @else {
				button type = "submit" { "Save Question" }
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
	user: User, theme: Theme, question: QuestionData, population: i32,
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

pub fn home_html(user: Option<User>, theme: Theme) -> String {
	let heading = "Home";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "The Equestrian Census, reimagined.";
	let mane = html! {
		h1 { "Census Consensus" }
		p { (description) }
		p {
			"Census Consensus is proud to announce that we have been entrusted"
			" to run this year's Equestrian Census! Twilight Sparkle has partnered"
			" with us to improve and revitalize the census for generations to come."
			" If you'd like to help with the census, please reach out to this year's"
			" event manager: "
			a href = ("https://www.fimfiction.net/user/237915/Silk+Rose") { "Silk Rose" sup { "↗" } }
			". She will help you get started and explain any questions you may have."
		}
		p {
			"The census is over, but you are welcome to explore the site,"
			" view all the chapters and questions, and try voting. To get started,"
			" create an account or sign in to an existing account by clicking the"
			" button below."
		}
		blockquote {
			"We do not store email addresses or API access tokens."
		}
		(button_link("Sign Up or Sign In", "/login/fimfic"))
	};
	let user_type = user.map(|user| user.user_type);
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, SITE_LINK, &theme))
		.header(header_html(user_type, Pages::Home, &theme))
		.mane(mane)
		.call()
}

pub fn about_html(user: Option<User>, theme: Theme, contributors: Vec<User>) -> String {
	let heading = "About";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "About this site and a contributor list.";
	let link = format!("{SITE_LINK}/about");
	let user_type = user.map(|user| user.user_type);
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		h2 { "Contributor List" }
		@for contributor in contributors.iter() {
			span class = "list-item" {
				span class = "row" {
					@if let Some(pfp_url) = &contributor.pfp_url {
						img src = (format!("{pfp_url}-64")) alt = (contributor.name) {}
						" - "
					}
				a href = (format!("https://www.fimfiction.net/user/{}/", contributor.id)) { (contributor.name) sup { "↗" } }
				}
			}
		}
	};
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, &link, &theme))
		.header(header_html(user_type, Pages::About, &theme))
		.mane(mane)
		.call()
}

pub fn question_preview_html(
	user: User, theme: Theme, question: Question, data: QuestionRevision,
	options: HashMap<String, f64>, population: i32,
) -> String {
	let heading = "Question Preview";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "Preview a question's outcomes.";
	let link = format!("{SITE_LINK}/questions/{}/preview", question.id);
	let opts = parse_options(
		&data.option_writing.clone().unwrap_or_default(),
		&data.question_type,
	);
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		h2 { "Question Preview" }
		(question_html(&question, &data, &opts))
		h2 { "Preview Selector" }
		h3 { (data.question_text) }
		form method = "get" action = (format!("/questions/{}/preview", question.id)) {
			@for (id, option) in &opts {
				span class = "row" {
					label for = (id) { (option) }
					(input_text_float_value_required(id, id, *options.get(id).unwrap_or(&0.0)))
				}
			}
			button type = "submit" { "Preview Outcome" }
		}
		@if !options.is_empty() {
			h2 { "Selected Preview" }
			@let options = OptionType::Percent(options);
			@let question_data = construct_question_data()
					.meta(question)
					.data(data)
					.option_texts(opts)
					.option_data(options)
					.population(population)
					.call();
			pre class = "left-text" {
				"DEBUG" br;
				(format!("{:#?}", &question_data.options))
			}
			@let (preview, errors) = result_formatter::format(&question_data);
			pre class = "left-text" {
				code {
					@for error in errors {
						"Error detected:" br; (error) br;
					}
				}
			}
			pre class = "left-text" {
				(PreEscaped (parse(&preview, &WarningType::Quiet)))
			}
		}
	};
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, &link, &theme))
		.header(header_html(Some(user.user_type), Pages::Questions, &theme))
		.mane(mane)
		.call()
}

pub fn question_html(
	question: &Question, data: &QuestionRevision, options: &Vec<(String, String)>,
) -> PreEscaped<String> {
	html! {
		p {
			(question.chapter_order.unwrap_or_default())
			". "
			(PreEscaped (parse(&data.question_text, &WarningType::Quiet)))
		}
		@if options.is_empty() {
			p { "No options found." }
		} @else {
			span class = (data.question_type) {
				@for (id, opt) in options {
					@let name = format!("{}-{id}", question.id);
					span class = "question-option row" {
						@if QuestionType::Multiselect == data.question_type {
							span { input id = (name) type = "checkbox" name = (question.id) value = (id) {} }
							label for = (name) { (PreEscaped (parse(opt, &WarningType::Quiet))) }
						} @else {
							span { input id = (name) type = "radio" name = (question.id) value = (id) {} }
							label for = (name) { (PreEscaped (parse(opt, &WarningType::Quiet))) }
						}
					}
				}
			}
		}
	}
}

pub fn chapter_survey_html(
	user: User, theme: Theme, chapter: ChapterRevision,
	questions: Vec<(Question, QuestionRevision)>,
) -> String {
	let heading = "Census Survey";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "Chapter survey page.";
	let link = format!("{SITE_LINK}/chapters/{}/survey", chapter.chapter_id);
	let user_type = user.user_type;
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		h2 { (chapter.title) }
		form method = "post" action = (format!("{SITE_LINK}/chapters/{}/submit", chapter.chapter_id)) {
			@for (question, data) in questions {
				@let opts = parse_options(
					&data.option_writing.clone().unwrap_or_default(),
					&data.question_type,
				);
				(question_html(&question, &data, &opts))
			}
			button type = "submit" { "Submit" }
		}
	};
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, &link, &theme))
		.header(header_html(Some(user_type), Pages::Chapters, &theme))
		.mane(mane)
		.call()
}

pub fn home_event_complete_html(user: User, theme: Theme) -> String {
	let heading = "Census Consensus";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "The Equestrian Census, redefined!";
	let link = format!("{SITE_LINK}/");
	let user_type = user.user_type.clone();
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		p { "Thank you for (potentially) participating in this year's event." }
		p {
			"The census is over, but you are welcome to explore the site,"
			" view all the chapters and questions, and try voting."
		}
	};
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, &link, &theme))
		.header(header_html(Some(user_type), Pages::Home, &theme))
		.mane(mane)
		.call()
}

pub fn chapter_preview_event_html(
	user: User, theme: Theme, chapter: ChapterRevision, text: &str,
) -> String {
	let heading = "Chapter Results (Event)";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "Chapter preview page.";
	let link = format!("{SITE_LINK}/chapters/{}/event-results", chapter.chapter_id);
	let user_type = user.user_type;
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		h2 { (chapter.title) }
		pre class = "left-text" {
			(PreEscaped (parse(text, &WarningType::Quiet)))
		}
	};
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, &link, &theme))
		.header(header_html(Some(user_type), Pages::Chapters, &theme))
		.mane(mane)
		.call()
}

pub fn chapter_preview_live_html(
	user: User, theme: Theme, chapter: ChapterRevision, text: &str,
) -> String {
	let heading = "Chapter Results (Live)";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "Chapter preview page.";
	let link = format!("{SITE_LINK}/chapters/{}/live-results", chapter.chapter_id);
	let user_type = user.user_type;
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		h2 { (chapter.title) }
		pre class = "left-text" {
			(PreEscaped (parse(text, &WarningType::Quiet)))
		}
	};
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, &link, &theme))
		.header(header_html(Some(user_type), Pages::Chapters, &theme))
		.mane(mane)
		.call()
}

pub fn chapter_preview_random_html(
	user: User, theme: Theme, chapter: ChapterRevision, text: &str,
) -> String {
	let heading = "Chapter Results (Random)";
	let title: String = format!("{heading} - {SITE_NAME}");
	let description = "Chapter preview page.";
	let link = format!("{SITE_LINK}/chapters/{}/results-random", chapter.chapter_id);
	let user_type = user.user_type;
	let mane = html! {
		h1 { (heading) }
		p { (description) }
		h2 { (chapter.title) }
		pre class = "left-text" {
			(PreEscaped (parse(text, &WarningType::Quiet)))
		}
	};
	html_builder()
		.theme(&theme)
		.head(head_html(&title, description, &link, &theme))
		.header(header_html(Some(user_type), Pages::Chapters, &theme))
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
		(icon_html(theme))
		(theme_color_html(theme))
		link rel = "canonical" href = (link);
		meta property = "og:title" content = (title);
		meta property = "og:description" content = (description);
		meta property = "og:image" content = { (format!("{SITE_LINK}/assets/cc-light-512.png")) };
		meta property = "og:url" content = (link);
		meta property = "og:type" content = "website";
		meta property = "og:site_name" content = (SITE_NAME);
		@let encode = encode_url(title);
		link
			crossorigin
			rel = "alternate"
			type = "application/json+oembed"
			href = { "https://census.silkrose.dev/oembed?" (encode) }
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
			 Theme::Light => meta name = "theme-color" content = "#f2d9e8";
			 Theme::Dark => meta name = "theme-color" content = "#aba4f4";
			 Theme::None => {
				meta name = "theme-color" content = "#f2d9e8";
				meta name = "theme-color" content = "#f2d9e8" media = "(prefers-color-scheme: light)";
				meta name = "theme-color" content = "#aba4f4" media = "(prefers-color-scheme: dark)";
			 },
		}
	}
}

fn icon_html(theme: &Theme) -> PreEscaped<String> {
	html! {
		@match theme {
			 Theme::Light => link rel = "icon" href = "/assets/cc-light.svg" type = "image/svg+xml";
			 Theme::Dark => link rel = "icon" href = "/assets/cc-dark.svg" type = "image/svg+xml";
			 Theme::None => {
				link rel = "icon" href = "/assets/cc-light.svg" type = "image/svg+xml"
				link rel = "icon" href = "/assets/cc-light.svg" type = "image/svg+xml" media = "(prefers-color-scheme: light)";
				link rel = "icon" href = "/assets/cc-dark.svg" type = "image/svg+xml" media = "(prefers-color-scheme: dark)";
			 },
		}
	}
}

fn header_html(user_type: Option<UserType>, page: Pages, theme: &Theme) -> PreEscaped<String> {
	html!(
		fieldset class = "logo" {
			input id = "census" type = "radio" name = "logo" {}
			label for = "census" { "Census" }
			input id = "consensus" type = "radio" name = "logo" {}
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
			@if let Some(user_type) = user_type {
				span class = "nav" {
						(header_link_html("/chapters", "Chapters", page == Pages::Chapters))
						(header_link_html("/questions", "Questions", page == Pages::Questions))
					@if user_type != UserType::Voter {
						(header_link_html("/feedback", "Feedback", page == Pages::Feedback))
					}
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
		button type = "button" onclick = (format!("window.location.href='{endpoint}';")) { (text) }
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
		a href = (format!("https://www.fimfiction.net/user/{}/", user.id)) { (user.name) sup { "↗" } }
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

fn options_explanation() -> PreEscaped<String> {
	let path = "./assets/options-explanation.md";
	let text = fs::read_to_string(path).unwrap();
	let markup = parse(&text, &WarningType::Quiet);
	html! {
		details open {
			summary { "Explanation: (Click to hide/show.)" }
			span class = "left-text" {
				(PreEscaped (markup))
			}
		}
	}
}

fn results_explanation() -> PreEscaped<String> {
	let path = "./assets/results-explanation.md";
	let text = fs::read_to_string(path).unwrap();
	let markup = parse(&text, &WarningType::Quiet);
	html! {
		details open {
			summary { "Explanation: (Click to hide/show.)" }
			span class = "left-text" {
				(PreEscaped (markup))
			}
		}
	}
}
