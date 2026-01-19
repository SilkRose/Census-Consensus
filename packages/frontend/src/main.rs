use leptos::prelude::*;
use leptos_mview::mview;

fn main() {
	std::panic::set_hook(Box::new(console_error_panic_hook::hook));
	leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
	mview! {
		div class="bg-purple-400" {
			"hi .3"
		}
	}
}
