use crate::asset_tag::AssetTag;
use crate::core::PotPath;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub struct DeepAsset {
	pub pot_path: PotPath,
	pub asset_tag: AssetTag,
}

impl DeepAsset {
	pub fn new(pot_path: &PotPath, asset_tag: &AssetTag) -> Self {
		DeepAsset {
			pot_path: pot_path.to_owned(),
			asset_tag: asset_tag.to_owned(),
		}
	}
	pub fn title(&self) -> String {
		let mut names = self.pot_path.segment_names();
		names.extend(vec![self.asset_tag.as_str().to_owned()]);
		let names = names.into_iter().filter(|it| !it.is_empty()).collect::<Vec<_>>();
		names.join("::")
	}
}

#[cfg(test)]
mod tests {
	use crate::asset_tag::AssetTag;
	use crate::core::deep_asset::DeepAsset;
	use crate::core::PotPath;

	#[test]
	fn title() {
		let pot_path = PotPath::CurrentFolder.extend("hunt");
		let asset_tag = AssetTag::equity("cost");
		let deep_asset = DeepAsset::new(&pot_path, &asset_tag);
		let title = deep_asset.title();
		assert_eq!(&title, "hunt::COST")
	}
}

