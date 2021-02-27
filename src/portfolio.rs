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
		let mut map = share_counts
			.into_iter()
			.map(|(asset, count)| market_value(asset, count, prices))
			.collect::<HashMap<AssetTag, _>>();
		map.insert(AssetTag::Usd, self.free_cash);
		map
	}
	pub fn market_value(&self, prices: &HashMap<AssetTag, f64>) -> f64 {
		self.market_values(prices)
			.into_iter()
			.map(|(_, value)| value)
			.sum()
	}
}

fn market_value(asset: AssetTag, count: f64, prices: &HashMap<AssetTag, f64>) -> (AssetTag, f64) {
	if count > 0.0 {
		let price = prices.get(&asset).cloned().expect("price");
		(asset, price * count)
	} else {
		(asset, 0.0)
	}
}
