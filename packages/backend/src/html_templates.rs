use maud::{DOCTYPE, html};

pub fn form_html_template() -> String {
	html! {
		(DOCTYPE) html lang = "en" {
			body {
				form method = "post" action = "/form-endpoint" {
					label for = "in1" { "test label 1" } br;
					input id = "in1" type = "text" name = "nm1" value = "test v1" {}
					br;
					label for = "in2" { "test label 2" } br;
					textarea id = "in2" type = "text" name = "nm2" value = "test v2" cols = "30" rows = "10" {}
					br;
					label { "radio label question" } br;
					input id = "in3" type = "radio" name = "radio1" value = "1" {}
					label for = "in3" { "test label 3" }
					input id = "in4" type = "radio" name = "radio1" value = "2" {}
					label for = "in4" { "test label 4" }
					br;
					label { "chbox label question" } br;
					input id = "in5" type = "checkbox" name = "chbox" value = "1" {}
					label for = "in5" { "test label 5" }
					input id = "in6" type = "checkbox" name = "chbox" value = "2" {}
					label for = "in6" { "test label 6" }
					br;
					select id = "dropdown" name = "drop1" {
						option value = "none" { "none" }
						option value = "op1" { "op1" }
						option value = "op2" { "op2" }
						option value = "op3" { "op3" }
					}
					br;
					button type = "submit" { "submit form" }
				}
			};
		};
	}
	.into()
}
