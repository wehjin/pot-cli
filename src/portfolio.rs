use std::collections::{HashMap, HashSet};

use crate::lot::Lot;

pub struct Portfolio { pub lots: Vec<Lot> }

impl Portfolio {
	pub fn symbols(&self) -> HashSet<String> {
		self.lots
			.iter()
			.map(Lot::symbol_string)
			.collect::<HashSet<_>>()
	}
	pub fn funded_symbols(&self) -> HashSet<String> {
		self.share_counts()
			.into_iter()
			.filter(|(_, count)| *count > 0.0)
			.map(|(symbol, _)| symbol)
			.collect()
	}
	pub fn share_counts(&self) -> HashMap<String, f64> {
		let mut map: HashMap<String, f64> = HashMap::new();
		for lot in &self.lots {
			let symbol = lot.asset_tag.as_str();
			let previous = map.get(symbol).cloned().unwrap_or(0.0);
			let next = previous + lot.share_count.as_f64();
			map.insert(symbol.to_string(), next);
		}
		map
	}
	pub fn market_values(&self, prices: &HashMap<String, f64>) -> HashMap<String, f64> {
		self.lots.iter().map(|lot| {
			let symbol = lot.symbol_string();
			if lot.is_funded() {
				let price = prices.get(&symbol).cloned().expect("price");
				(symbol, price * lot.share_count.as_f64())
			} else {
				(symbol, 0.0)
			}
		}).collect()
	}
}
