use std::collections::{BTreeSet, HashMap, HashSet};
use std::error::Error;

pub use deep_asset::*;
pub use pot_path::*;

use crate::asset_tag::AssetTag;
use crate::portfolio::Portfolio;
use crate::pot::{FolderPot, Pot};

mod pot_path;
mod deep_asset;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Ramp { Golden, Flat }

impl Ramp {
	pub fn from_str(s: &str) -> Self {
		let ramp = match s.to_lowercase().trim() {
			"golden" => Ramp::Golden,
			"flat" => Ramp::Flat,
			_ => Ramp::Golden
		};
		ramp
	}
	pub fn as_f64(&self) -> f64 {
		match self {
			Ramp::Golden => 1.618f64,
			Ramp::Flat => 1.0,
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Ramp::Golden => "golden",
			Ramp::Flat => "flat",
		}
	}

	pub fn pow_weight(&self, pos: usize) -> f64 { self.as_f64().powf(pos as f64) }
}


#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct AssetGroup {
	pub tag: AssetTag,
	pub assets: BTreeSet<DeepAsset>,
}

impl AssetGroup {
	pub fn new(asset: &DeepAsset) -> Self {
		let mut assets = BTreeSet::new();
		assets.insert(asset.clone());
		AssetGroup { tag: asset.asset_tag.clone(), assets }
	}
	pub fn add(&self, asset: &DeepAsset) -> Self {
		assert_eq!(asset.asset_tag, self.tag);
		let mut assets = self.assets.clone();
		assets.insert(asset.clone());
		AssetGroup { tag: self.tag.clone(), assets }
	}
	pub fn market_value(&self, prices: &HashMap<AssetTag, f64>) -> Result<f64, Box<dyn Error>> {
		let mut sum = 0.0;
		for asset in &self.assets {
			let pot = FolderPot::from_pot_path(&asset.pot_path);
			let portfolio = Portfolio { lots: pot.read_lots()?, free_cash: 0.0 };
			let values = portfolio.market_values(prices);
			let value = values.get(&asset.asset_tag).expect("value");
			sum += *value;
		}
		Ok(sum)
	}
}

pub fn into_groups(assets: HashSet<DeepAsset>) -> HashSet<AssetGroup> {
	let mut groups: HashMap<AssetTag, AssetGroup> = HashMap::new();
	for asset in assets {
		let group = {
			let group = groups.get(&asset.asset_tag);
			match group {
				None => AssetGroup::new(&asset),
				Some(group) => group.add(&asset),
			}
		};
		groups.insert(asset.asset_tag.clone(), group);
	}
	groups.values().into_iter().cloned().collect()
}