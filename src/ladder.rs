use std::collections::{HashMap, HashSet};

use crate::asset_tag::AssetTag;
use crate::core::Ramp;

#[derive(Debug)]
pub struct Ladder {
	pub targets: Vec<AssetTag>,
	pub ramp: Ramp,
}

impl Ladder {
	pub fn target_symbols_ascending(&self) -> Vec<AssetTag> {
		self.targets.clone()
	}
	pub fn target_symbols_descending(&self) -> Vec<AssetTag> {
		self.target_symbols_ascending()
			.into_iter()
			.rev()
			.collect()
	}
	pub fn target_symbols(&self) -> HashSet<AssetTag> {
		self.target_symbols_ascending().into_iter().collect()
	}
	pub fn target_weights(&self) -> HashMap<AssetTag, f64> {
		self.targets
			.iter()
			.enumerate()
			.map(|(i, asset_type)| (asset_type.clone(), self.ramp.pow_weight(i)))
			.collect::<HashMap<AssetTag, _>>()
	}
	pub fn target_portions(&self) -> HashMap<AssetTag, f64> {
		let weights = self.target_weights();
		let full_weight: f64 = weights.values().sum();
		weights.iter()
			.map(|(asset_tag, weight)| (asset_tag.clone(), *weight / full_weight))
			.collect::<HashMap<AssetTag, _>>()
	}
}
