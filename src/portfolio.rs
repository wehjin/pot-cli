use std::collections::{HashMap, HashSet};

use crate::lot::Lot;

pub struct Portfolio {
	pub lots: Vec<Lot>,
	pub free_cash: f64,
}

impl Portfolio {
	pub fn symbols(&self) -> HashSet<String> {
		let mut set = self.lots
			.iter()
			.map(Lot::symbol_string)
			.collect::<HashSet<_>>();
		set.insert("USD".to_string());
		set
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
		map.insert("USD".to_string(), self.free_cash);
		map
	}
	pub fn market_values(&self, prices: &HashMap<String, f64>) -> HashMap<String, f64> {
		let mut map: HashMap<String, f64> = self.lots.iter().map(|lot| {
			let symbol = lot.symbol_string();
			if lot.is_funded() {
				let price = prices.get(&symbol).cloned().expect("price");
				(symbol, price * lot.share_count.as_f64())
			} else {
				(symbol, 0.0)
			}
		}).collect();
		map.insert("USD".to_string(), self.free_cash);
		map
	}
	pub fn market_value(&self, prices: &HashMap<String, f64>) -> f64 {
		let market_values = self.market_values(&prices);
		market_values.iter().map(|(_, value)| *value).sum()
	}
}
