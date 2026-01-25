use maud::{DOCTYPE, html};

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
					input type = "text" id = "id" name = "id" inputmode = "numeric" pattern = r"\d*" minlength = "1" maxlength = "8" required {  }
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
					input type = "text" id = "id" name = "id" inputmode = "numeric" pattern = r"\d*" minlength = "1" maxlength = "8" required {  }
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
