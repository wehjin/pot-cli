use std::collections::{HashMap, HashSet};
use std::error::Error;

use smarket::yf::PricingResult;

use crate::{disk, Ladder, Lot};

pub fn status(ladder: Ladder) -> Result<(), Box<dyn Error>> {
	let lots = disk::read_lots()?;
	let portions = ladder.portions();
	let counts = counts(&lots);
	let price_symbols = {
		let mut symbols = portions.keys().cloned().collect::<HashSet<_>>();
		symbols.extend(counts.keys().cloned().collect::<HashSet<_>>());
		symbols.into_iter().collect::<Vec<_>>()
	};
	let prices = smarket::yf::price_assets(&price_symbols)?
		.iter()
		.map(|(symbol, result)| {
			let usd_price = match result {
				PricingResult::Priced { usd_price, .. } => *usd_price,
				_ => panic!("missing price")
			};
			(symbol.to_string(), usd_price.as_f64())
		})
		.collect::<HashMap<String, _>>();
	let values = prices.iter().map(|(symbol, price)| {
		let count = counts.get(symbol).cloned().unwrap_or(0.0);
		(symbol.to_string(), price * count)
	}).collect::<HashMap<String, _>>();
	let full_value: f64 = values.values().sum();

	println!(
		"{:8}  {:6}    {:10}  {:^6}    {:10}  {:^6}    {:10}  {:^6}",
		"ASSET ID", "SHARES",
		"MARKET($)", "%",
		"TARGET($)", "%",
		"DRIFT($)", "%"
	);
	for symbol in ladder.ordered_symbols() {
		let target_portion = portions.get(&symbol).expect("portion");
		let count = counts.get(&symbol).cloned().unwrap_or(0.0);
		let market = values.get(&symbol).expect("value");
		let market_portion = market / full_value;
		let target = target_portion * full_value;
		let drift = market - target;
		let drift_portion = market_portion - target_portion;
		println!(
			"{:8}  {:6.2}    {:10.1}  {:5.1}%    {:10.1}  {:5.1}%    {:10.1}  {:5.1}% ",
			symbol, count,
			market, market_portion * 100.0,
			target, target_portion * 100.0,
			drift, drift_portion * 100.0
		)
	}
	Ok(())
}

pub fn lots() -> Result<(), Box<dyn Error>> {
	let lots = disk::read_lots()?;
	for lot in lots {
		println!("{:?}", lot);
	}
	Ok(())
}

pub fn init() -> Result<(), Box<dyn Error>> {
	disk::init();
	Ok(())
}

fn counts(lots: &Vec<Lot>) -> HashMap<String, f64> {
	let mut map: HashMap<String, f64> = HashMap::new();
	for lot in lots {
		let symbol = lot.asset_tag.as_str();
		let previous = map.get(symbol).cloned().unwrap_or(0.0);
		let next = previous + lot.share_count.as_f64();
		map.insert(symbol.to_string(), next);
	}
	map
}
