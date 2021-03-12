#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub enum PotPath {
	CurrentFolder,
	SubFolder(Box<PotPath>, String),
}

impl PotPath {
	pub fn segment_names(&self) -> Vec<String> {
		match self {
			PotPath::CurrentFolder => vec!["".to_string()],
			PotPath::SubFolder(pred, name) => {
				let mut names = pred.segment_names();
				names.extend(vec![name.to_string()]);
				names
			}
		}
	}
	pub fn extend(&self, name: &str) -> Self {
		PotPath::SubFolder(Box::new(self.clone()), name.to_owned())
	}
}

#[cfg(test)]
mod tests {
	use crate::core::pot_path::PotPath;

	#[test]
	fn extendable() {
		let path = PotPath::CurrentFolder.extend("a").extend("b");
		assert_eq!(path.segment_names(), vec!["".to_string(), "a".into(), "b".into()]);
	}
}
