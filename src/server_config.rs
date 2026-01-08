use actix_web::web::Data;

pub type FimficData = Data<Fimfic>;

pub struct Fimfic {
	pub client_id: Box<str>,
	pub client_secret: Box<str>,
	pub oauth_redirect_url: Box<str>
}
