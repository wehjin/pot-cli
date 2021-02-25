use std::fs::File;
use std::io;
use std::path::PathBuf;

pub struct FolderPot {
	path: PathBuf
}

impl FolderPot {
	pub fn new() -> Self {
		let path = PathBuf::from(".");
		FolderPot { path }
	}
	pub fn create_team_file(&self) -> io::Result<File> { File::create(self.team_file()) }
	pub fn open_team_file(&self) -> io::Result<File> { File::open(self.team_file()) }
	fn team_file(&self) -> PathBuf { self.path.join(TEAM_TXT) }
}

const TEAM_TXT: &str = "team.txt";
