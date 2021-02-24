use std::collections::{HashMap, HashSet};

use crate::AssetTag;
use crate::core::Ramp;

#[derive(Debug)]
pub struct Ladder {
	pub targets: Vec<AssetTag>,
	pub ramp: Ramp,
}

impl Ladder {
	pub fn target_symbols_ascending(&self) -> Vec<String> {
		self.targets
			.iter()
			.map(|it| it.as_str().to_uppercase())
			.collect()
	}
	pub fn target_symbols_descending(&self) -> Vec<String> {
		self.target_symbols_ascending()
			.into_iter()
			.rev()
			.collect()
	}
	pub fn target_symbols(&self) -> HashSet<String> {
		self.target_symbols_ascending().into_iter().collect()
	}
	pub fn target_weights(&self) -> HashMap<String, f64> {
		self.targets
			.iter()
			.enumerate()
			.map(|(i, asset_type)| {
				let symbol = asset_type.as_str();
				(symbol.to_uppercase(), self.ramp.pow_weight(i))
			})
			.collect::<HashMap<String, _>>()
	}
	pub fn target_portions(&self) -> HashMap<String, f64> {
		let weights = self.target_weights();
		let full_weight: f64 = weights.values().sum();
		weights.iter()
			.map(|(symbol, weight)| (symbol.to_string(), *weight / full_weight))
			.collect::<HashMap<String, _>>()
	}
}
