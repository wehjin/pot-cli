use std::collections::{HashMap, HashSet};

use crate::asset_tag::AssetTag;
use crate::lot::Lot;

pub struct Portfolio {
	pub lots: Vec<Lot>,
	pub free_cash: f64,
}

impl Portfolio {
	pub fn symbols(&self) -> HashSet<AssetTag> {
		let mut set = self.lots
			.iter()
			.map(Lot::symbol_string)
			.collect::<HashSet<_>>();
		set.insert(AssetTag::Usd);
		set
	}
	pub fn funded_symbols(&self) -> HashSet<AssetTag> {
		self.share_counts()
			.into_iter()
			.filter(|(_, count)| *count > 0.0)
			.map(|(asset_tag, _)| asset_tag)
			.collect()
	}
	pub fn share_counts(&self) -> HashMap<AssetTag, f64> {
		let mut map: HashMap<AssetTag, f64> = HashMap::new();
		for lot in &self.lots {
			let asset_tag = &lot.asset_tag;
			let previous = map.get(asset_tag).cloned().unwrap_or(0.0);
			let next = previous + lot.share_count.as_f64();
			map.insert(asset_tag.clone(), next);
		}
		map.insert(AssetTag::Usd, self.free_cash);
		map
	}
	pub fn market_values(&self, prices: &HashMap<AssetTag, f64>) -> HashMap<AssetTag, f64> {
		let share_counts = self.share_counts();
		let mut map = share_counts.into_iter().map(|(symbol, count)| {
			if count > 0.0 {
				let price = prices.get(&symbol).cloned().expect("price");
				(symbol, price * count)
			} else {
				(symbol, 0.0)
			}
		}).collect::<HashMap<AssetTag, _>>();
		map.insert(AssetTag::Usd, self.free_cash);
		map
	}
}
