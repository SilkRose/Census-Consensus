pub struct FimficCfg {
	pub client_id: Box<str>,
	pub client_secret: Box<str>,
	pub oauth_redirect_url: Box<str>,
	/// Login URL except missing state (ie. `format!("{url}&state={state}")` to
	/// get a complete URL)
	pub login_url: Box<str>
}

/// Makes a login url, purposefully without scope so we can reuse this and
/// clients can generate their own scope to put on it
pub fn make_login_url(client_id: &str, oauth_redirect_url: &str) -> Box<str> {
	format!("https://www.fimfiction.net/authorize-app?client_id={client_id}&response_type=code&scope=&redirect_uri={oauth_redirect_url}")
		.into_boxed_str()
}

// todo need real header
pub const FIMFIC_USER_AGENT: &str = "fish";
pub const FIMFIC_NO_PFP_LINK: &str = "https://static.fimfiction.net/images/none_64.png";
