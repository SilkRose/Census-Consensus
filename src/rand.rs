use base64ct::{ Base64Url, Encoding as _ };
use rand::RngCore as _;
use rand::rngs::{ ReseedingRng, OsRng };
use rand_chacha::ChaCha20Core;
use std::cell::RefCell;

thread_local! {
	static RNG: RefCell<ReseedingRng<ChaCha20Core, OsRng>> = {
		let rng = ReseedingRng::new(64_000, OsRng).unwrap();
		RefCell::new(rng)
	};
}

pub fn gen_auth_state() -> String {
	let mut bytes = [0u8; 96];
	RNG.with_borrow_mut(|rng| rng.fill_bytes(&mut bytes));
	Base64Url::encode_string(&bytes)
}
