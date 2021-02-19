use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;

use smarket::yf::PricingResult;

use crate::{AssetTag, Custodian, disk, Ladder, Lot, ShareCount};

pub fn add_lot(custody: &str, symbol: &str, share_count: f64, uid: Option<u64>) -> Result<(), Box<dyn Error>> {
	let uid = uid.unwrap_or_else(Lot::random_uid);
	let symbol = &symbol.to_uppercase();
	let mut lots = disk::read_lots()?;
	let existing = lots.iter().find(|it| it.uid == uid);
	if existing.is_some() {
		println!("skip: Lot {:016} already exists", uid)
	} else {
		let lot = Lot {
			custodian: Custodian(custody.to_string()),
			asset_tag: AssetTag(symbol.to_string()),
			share_count: ShareCount(share_count),
			uid,
		};
		lots.extend(vec![lot]);
		disk::write_lots(&lots)?;
		println!("{:016}", uid);
	}
	Ok(())
}

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
		"{:8}  {:9}    {:10}  {:^6}    {:10}  {:^6}    {:10}  {:^6}",
		"ASSET ID", "SHARES", "MARKET($)", "%PF", "TARGET($)", "%PF", "DRIFT($)", "%PF"
	);
	for symbol in ladder.ordered_symbols() {
		let target_portion = portions.get(&symbol).expect("portion");
		let count = counts.get(&symbol).cloned().unwrap_or(0.0);
		let market = values.get(&symbol).expect("value").clone();
		let market_portion = market / full_value;
		let target = target_portion * full_value;
		let drift = market - target;
		let drift_portion = market_portion - target_portion;
		println!(
			"{:8}  {:>9.2}    {:>10}  {:5.1}%    {:>10}  {:5.1}%    {:>10}  {:5.1}% ",
			symbol, count,
			shorten(market), market_portion * 100.0,
			shorten(target), target_portion * 100.0,
			shorten(drift), drift_portion * 100.0
		)
	}
	Ok(())
}

pub fn lots() -> Result<(), Box<dyn Error>> {
	println!("{:16}  {:10}  {:8}  {:8}", "LOT ID", "CUSTODY", "SYMBOL", "COUNT");
	let lots = disk::read_lots()?;
	for lot in lots {
		println!(
			"{:016x}  {:10}  {:8}  {:8}",
			lot.uid, lot.custodian.as_str(), lot.asset_tag.as_str(), lot.share_count.as_f64()
		);
	}
	Ok(())
}

pub fn init() -> Result<(), Box<dyn Error>> {
	let current_folder = env::current_dir()?;
	if disk::is_not_initialized() {
		disk::init()?;
		println!("Initialized empty Pot in {}", current_folder.display());
	} else {
		println!("Skipped reinitializing existing Pot in {}", current_folder.display());
	}
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

pub fn shorten(no: f64) -> String {
	if no.is_nan() {
		"$NAN".to_string()
	} else if no == 0.0 {
		"$0".to_string()
	} else {
		let pos = no.abs();
		let quantity = if pos >= 1e12 {
			"1.0T+".to_string()
		} else {
			let (short_pos, unit) = if pos >= 1e9 {
				(pos / 1e9, "B")
			} else if pos >= 1e6 {
				(pos / 1e6, "M")
			} else if pos >= 1e3 {
				(pos / 1e3, "K")
			} else {
				(pos, "")
			};
			let s = format!("{:07.3}", short_pos);
			let digits = if short_pos >= 100.0 {
				&s[..3]
			} else if short_pos >= 10.0 {
				&s[1..5]
			} else if short_pos >= 1.0 {
				&s[2..6]
			} else {
				&s[3..]
			};
			format!("{}{}", digits, unit)
		};
		if no.is_sign_negative() {
			format!("(${})", quantity)
		} else {
			format!("${}", quantity)
		}
	}
}