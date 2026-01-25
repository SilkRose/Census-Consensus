use maud::{DOCTYPE, html};

pub fn ban_user_html() -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				form method = "post" action = "/ban-user" {
					label for = "id" { "User ID:" }
					br;
					input type = "number" id = "id" name = "id" inputmode = "numeric" pattern= "[0-9]+" min = "1" required {  }
					br;
					label for = "reason" { "Ban Reason:" }
					br;
					textarea type = "text" id = "reason" name = "reason" minlength = "8" maxlength = "256" rows = "4" cols = "40" required {}
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
					@if let Some(public_feedback) = public_feedback {
							textarea id = "public" type = "text" name = "feedback_public" maxlength = "1000000" cols = "30" rows = "10" { (public_feedback) }
					} @else {
						textarea id = "public" type = "text" name = "feedback_public" maxlength = "1000000" cols = "30" rows = "10" {}
					}
					br;
					label for = "private" { h3  { "Private Feedback" } }
					br;
					p style = "opacity: 80%" { "Shared only with the developers and writers of this event." }
					@if let Some(private_feedback) = private_feedback {
							textarea id = "private" type = "text" name = "feedback_private" maxlength = "1000000" cols = "30" rows = "10" { (private_feedback) }
					} @else {
						textarea id = "private" type = "text" name = "feedback_private" maxlength = "1000000" cols = "30" rows = "10" {}
					}
					br;
					button type = "submit" { "Submit Feedback" }
				}
			};
		};
	}
	.into()
}
