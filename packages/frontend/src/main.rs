use leptos::prelude::*;
use leptos_mview::mview;
use leptos_meta::{ Stylesheet, Title, provide_meta_context };
use leptos_router::path;
use leptos_router::components::{ Route, Router, Routes };

fn main() {
	std::panic::set_hook(Box::new(console_error_panic_hook::hook));
	leptos::mount::mount_to_body(App);
}

#[component]
pub fn App() -> impl IntoView {
	provide_meta_context();

	mview! {
		Stylesheet id="leptos" href="/_/april-fools-2026.css";
		Title text="Welcome to Leptos";

		Router {
			main class="bg-purple-400" {
				Routes fallback=["not found"] {
					Route path={ path!("/") } view={ HomePage };
					Route path={ path!("/*any") } view={ NotFound };
				}
			}
		}
	}
}

#[component]
fn HomePage() -> impl IntoView {
	let count = RwSignal::new(0);
	let on_click = move |_| *count.write() += 1;

	mview! {
		h1 { "Welcome to Leptos!" }
		button on:click={ on_click } {
			"Click Me: " { count }
		}
		br;

		a
			rel="external"
			class="underline text-green-400"
			href="/login/fimfic"
		{ "evil whimsical login button" }
	}
}

#[component]
fn NotFound() -> impl IntoView {
	#[cfg(feature = "ssr")] {
		use actix_web::http::StatusCode;
		use leptos_actix::ResponseOptions;

		let resp = expect_context::<ResponseOptions>();
		resp.set_status(StatusCode::NOT_FOUND);
	}

	mview! {
		h1 { "Not Found" }
	}
}
