use maud::{DOCTYPE, html};

pub fn form_html_template() -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				form method = "post" action = "/user-feedback" {
					label for = "public" { h2  { "Public Feedback" } } br;
					textarea id = "public" type = "text" name = "feedback_public" cols = "30" rows = "10" {}
					br;
					label for = "private" { h2  { "Private Feedback" } } br;
					textarea id = "private" type = "text" name = "feedback_private" cols = "30" rows = "10" {}
					br;
					button type = "submit" { "submit form" }
				}
			};
		};
	}
	.into()
}
