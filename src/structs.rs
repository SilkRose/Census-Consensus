use std::fmt;

#[derive(Debug, Clone)]
pub enum Table {
	Users,
	Tokens,
	BannedUsers,
	Chapters,
	Writings,
	Questions,
	Options,
	Votes,
	StoryUpdates,
}

impl fmt::Display for Color {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let text = match self {
			Table::Users => "Users",
			Table::Tokens => "Tokens",
			Table::BannedUsers => "Banned_users",
			Table::Chapters => "Chapters",
			Table::Writings => "Writings",
			Table::Questions => "Questions",
			Table::Options => "Options",
			Table::Votes => "Votes",
			Table::StoryUpdates => "Story_updates",
		};
		write!(f, "{text}")
	}
}
