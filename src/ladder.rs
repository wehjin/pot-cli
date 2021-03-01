use std::collections::{HashMap, HashSet};

use crate::asset_tag::AssetTag;
use crate::core::Ramp;

#[derive(Debug)]
pub struct Ladder {
	pub targets: Vec<AssetTag>,
	pub ramp: Ramp,
}

impl Ladder {
	pub fn asset_portions(&self) -> Vec<(AssetTag, f64)> {
		let portions = self.target_portions();
		self.targets.iter().map(|asset| {
			let portion = match portions.get(asset) {
				Some(portion) => portion.to_owned(),
				None => 0.0,
			};
			(asset.to_owned(), portion)
		}).collect::<Vec<_>>()
	}
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
	pub fn promote_target(&mut self, asset: &AssetTag) -> Option<usize> {
		let position = self.targets.iter().position(|it| it == asset);
		match position {
			Some(position) => {
				self.targets.remove(position);
				let new_position = (position + 1).min(self.targets.len());
				self.targets.insert(new_position, asset.to_owned());
				Some(new_position)
			}
			None => None
		}
	}
	pub fn demote_target(&mut self, asset: &AssetTag) -> Option<usize> {
		let position = self.targets.iter().position(|it| it == asset);
		match position {
			Some(position) => {
				self.targets.remove(position);
				let new_position = position.max(1) - 1;
				self.targets.insert(new_position, asset.to_owned());
				Some(new_position)
			}
			None => None
		}
	}
}
