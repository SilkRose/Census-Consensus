use maud::{DOCTYPE, html};

pub fn form_html_template(
	private_feedback: Option<String>, public_feedback: Option<String>,
) -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				form method = "post" action = "/user-feedback" {
					label for = "public" { h2  { "Public Feedback" } } br;
					@if let Some(public_feedback) = public_feedback {
							textarea id = "public" type = "text" name = "feedback_public" cols = "30" rows = "10" { (public_feedback) }
					} @else {
						textarea id = "public" type = "text" name = "feedback_public" cols = "30" rows = "10" {}
					}
					br;
					label for = "private" { h2  { "Private Feedback" } } br;
					@if let Some(private_feedback) = private_feedback {
							textarea id = "private" type = "text" name = "feedback_private" cols = "30" rows = "10" { (private_feedback) }
					} @else {
						textarea id = "private" type = "text" name = "feedback_private" cols = "30" rows = "10" {}
					}
					br;
					button type = "submit" { "submit form" }
				}
			};
		};
	}
	.into()
}
