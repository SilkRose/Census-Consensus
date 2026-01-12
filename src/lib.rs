use leptos::prelude::*;

use leptos_mview::mview;
use leptos_meta::{ Stylesheet, Title, provide_meta_context };
use leptos_router::path;
use leptos_router::components::{ Route, Router, Routes };

#[cfg(feature = "ssr")]
pub mod auth;
#[cfg(feature = "ssr")]
pub mod database;
#[cfg(feature = "ssr")]
pub mod env_vars;
#[cfg(feature = "ssr")]
pub mod fimfic_cfg;
#[cfg(feature = "ssr")]
pub mod http;
#[cfg(feature = "ssr")]
pub mod rand;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
	std::panic::set_hook(Box::new(console_error_panic_hook::hook));
	leptos::mount::hydrate_body(App);
}

pub fn shell(options: &LeptosOptions) -> impl IntoView + use<> {
	mview! {
		!DOCTYPE html;
		html lang="en" {
			head {
				meta charset="utf-8";
				meta name="viewport" content="width=device-width, initial-scale=1";
				AutoReload options={ options.clone() };
				HydrationScripts options={ options.clone() };
				leptos_meta::MetaTags;
			}

			body {
				App;
			}
		}
	}
}

#[component]
pub fn App() -> impl IntoView {
	provide_meta_context();

	mview! {
		Stylesheet id="leptos" href="/_/april-fools-2026.css";
		Title text="Welcome to Leptos";

		Router {
			main class="bg-green-400" {
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
			class="underline text-purple-400"
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
