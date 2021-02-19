use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;

use smarket::yf::PricingResult;

use crate::{AssetTag, Custodian, disk, Ladder, Lot, Portfolio, ShareCount};

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
	let portfolio = Portfolio { lots: disk::read_lots()? };
	let off_target_symbols = portfolio.symbols().difference(&ladder.target_symbols()).cloned().collect::<HashSet<_>>();
	let mut portion_targets = ladder.target_portions();
	for off_target_symbol in &off_target_symbols {
		portion_targets.insert(off_target_symbol.clone(), 0.0);
	}
	let lot_counts = portfolio.share_counts();
	let prices = smarket::yf::price_assets(&portfolio.funded_symbols().into_iter().collect())?
		.iter()
		.map(|(symbol, result)| {
			let usd_price = match result {
				PricingResult::Priced { usd_price, .. } => *usd_price,
				_ => panic!("missing price")
			};
			(symbol.to_string(), usd_price.as_f64())
		})
		.collect::<HashMap<String, _>>();
	let mut actual_values = portfolio.market_values(&prices);
	for ref target_symbol in ladder.target_symbols() {
		if !actual_values.contains_key(target_symbol) {
			actual_values.insert(target_symbol.clone(), 0.0);
		}
	}
	let full_value: f64 = actual_values.values().sum();
	println!(
		"{:8}  {:9}    {:10}  {:^6}    {:10}  {:^6}    {:10}  {:^6}",
		"ASSET ID", "SHARES", "MARKET($)", "%PF", "TARGET($)", "%PF", "DRIFT($)", "%PF"
	);
	let ordered_symbols = {
		let mut symbols = ladder.target_symbols_descending();
		let mut ordered_off_target_symbols = off_target_symbols.iter().cloned().collect::<Vec<_>>();
		ordered_off_target_symbols.sort();
		symbols.extend(ordered_off_target_symbols);
		symbols
	};
	for symbol in ordered_symbols {
		let target_portion = portion_targets.get(&symbol).expect("portion");
		let count = lot_counts.get(&symbol).cloned().unwrap_or(0.0);
		let market = actual_values.get(&symbol).expect("value").clone();
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
	// TODO: Display low percentages s <0.1% instead of 0%)
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