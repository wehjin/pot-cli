use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;

use smarket::yf::PricingResult;

use crate::{Custodian, disk, Lot, Portfolio, print, ShareCount};
use crate::asset_tag::AssetTag;
use crate::core::Ramp;
use crate::pot::FolderPot;

pub fn set_cash(value: f64) -> Result<(), Box<dyn Error>> {
	disk::write_cash(value)
}

pub fn cash() -> Result<(), Box<dyn Error>> {
	let cash_value = disk::read_cash()?;
	println!("${:.2}", cash_value);
	Ok(())
}

pub fn set_ramp(ramp_s: &str) -> Result<(), Box<dyn Error>> {
	let ramp = Ramp::from_str(ramp_s);
	disk::write_ramp(ramp)?;
	println!("{}", ramp_s);
	Ok(())
}

pub fn ramp() -> Result<(), Box<dyn Error>> {
	let ramp = disk::read_ramp()?;
	println!("{}", ramp.as_str());
	Ok(())
}

pub fn targets() -> Result<(), Box<dyn Error>> {
	let pot = FolderPot::new();
	let targets = disk::read_targets(&pot)?;
	print::title("TARGETS");
	targets.iter().for_each(|s| { println!("{}", s); });
	Ok(())
}

pub fn add_targets(symbols: &str) -> Result<(), Box<dyn Error>> {
	let pot = FolderPot::new();
	let symbols = symbols
		.split(",")
		.map(|s| s.trim().to_uppercase())
		.collect::<Vec<_>>();
	let mut targets = disk::read_targets(&pot)?;
	let mut added = Vec::new();
	symbols.iter().rev().for_each(|symbol| {
		let position = targets.iter().position(|t| t == symbol);
		if position.is_none() {
			targets.insert(0, symbol.to_string());
			added.insert(0, symbol.to_string());
		}
	});
	if !added.is_empty() {
		disk::write_targets(&targets, &pot)?;
	}
	println!("{}", added.join(","));
	Ok(())
}

pub fn shares(custodian: &str, symbol: &str, count: Option<f64>) -> Result<(), Box<dyn Error>> {
	match count {
		None => {
			let count = disk::read_shares(&custodian, &symbol)?;
			println!("{}", count);
		}
		Some(count) => {
			let uid = disk::write_shares(&custodian, &symbol, count)?;
			println_uid(uid);
		}
	}
	Ok(())
}

pub fn add_lot(custody: &str, asset_tag: &str, share_count: f64, uid: Option<u64>) -> Result<(), Box<dyn Error>> {
	let uid = uid.unwrap_or_else(Lot::random_uid);
	let mut lots = disk::read_lots()?;
	let existing = lots.iter().find(|it| it.uid == uid);
	if existing.is_some() {
		println!("skip: Lot {:016} already exists", uid)
	} else {
		let lot = Lot {
			custodian: Custodian(custody.to_string()),
			asset_tag: asset_tag.into(),
			share_count: ShareCount(share_count),
			uid,
		};
		lots.extend(vec![lot]);
		disk::write_lots(&lots)?;
		println_uid(uid);
	}
	Ok(())
}

fn println_uid(uid: u64) {
	println!("{:016}", uid);
}

pub fn value() -> Result<(), Box<dyn Error>> {
	let portfolio = disk::read_portfolio()?;
	let prices = fetch_price_map(&portfolio)?;
	let value = portfolio.market_value(&prices);
	println!("{}", shorten_dollars(value));
	Ok(())
}

pub fn status() -> Result<(), Box<dyn Error>> {
	let pot = FolderPot::new();
	let ladder = disk::read_ladder(&pot)?;
	let portfolio = disk::read_portfolio()?;
	println!("Free Cash: {}", shorten_dollars(portfolio.free_cash));
	let off_target_symbols = {
		let mut set = portfolio.symbols().difference(&ladder.target_symbols()).cloned().collect::<HashSet<_>>();
		set.insert(AssetTag::Usd);
		set
	};
	let portion_targets = {
		let mut portion_targets = ladder.target_portions();
		for off_target_symbol in &off_target_symbols {
			portion_targets.insert(off_target_symbol.clone(), 0.0);
		}
		portion_targets
	};
	let lot_counts = portfolio.share_counts();
	let symbol_prices = fetch_price_map(&portfolio)?;
	let mut market_values = portfolio.market_values(&symbol_prices);
	for ref target_symbol in ladder.target_symbols() {
		if !market_values.contains_key(target_symbol) {
			market_values.insert(target_symbol.clone(), 0.0);
		}
	}
	let full_value: f64 = market_values.values().sum();
	println!(
		"{:8}  {:9}    {:10}  {:^6}    {:^11}  {:^6}    {:10}",
		"ASSET ID", "SHARES", "MARKET($)", "%PF", "TARGET(%PF)", "$", "ACTION($)"
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
		let market = market_values.get(&symbol).expect("value").clone();
		let market_portion = market / full_value;
		let target = target_portion * full_value;
		let drift = market - target;
		println!(
			"{:8}  {:>9.2}    {:>10}  {:5.1}%    {:10.1}%  {:>6}    {:>10}",
			symbol.as_str(), count,
			shorten_dollars(market), market_portion * 100.0,
			target_portion * 100.0, shorten_dollars(target),
			shorten_dollars_delta(-drift)
		)
	}
	// TODO: Display low percentages as <0.1% instead of 0%)
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

pub fn lots_symbols() -> Result<(), Box<dyn Error>> {
	let lots = disk::read_lots()?;
	let unique_symbols = lots
		.iter()
		.filter(|lot| lot.share_count.as_f64() > 0.0)
		.map(|lot| lot.asset_tag.as_str().to_string())
		.collect::<HashSet<_>>();
	let sorted_symbols = {
		let mut v = unique_symbols.into_iter().collect::<Vec<_>>();
		v.sort();
		v
	};
	let line: String = sorted_symbols.join(",");
	println!("{}", line);
	Ok(())
}

pub fn init() -> Result<(), Box<dyn Error>> {
	let current_folder = env::current_dir()?;
	if disk::is_not_initialized() {
		disk::init()?;
		println!("Initialized pot in {}", current_folder.display());
	} else {
		println!("Skipped reinitializing existing pot in {}", current_folder.display());
	}
	Ok(())
}

pub fn shorten_dollars(no: f64) -> String {
	if no.is_nan() {
		"$NAN".to_string()
	} else if no == 0.0 {
		"$0".to_string()
	} else if no.is_sign_negative() {
		format!("(${})", shorten_abs(no))
	} else {
		format!("${}", shorten_abs(no))
	}
}

pub fn shorten_dollars_delta(no: f64) -> String {
	if no.is_nan() {
		"$NAN".to_string()
	} else if no == 0.0 {
		"=$0".to_string()
	} else if no.is_sign_negative() {
		format!("-${}", shorten_abs(no))
	} else {
		format!("+${}", shorten_abs(no))
	}
}

fn shorten_abs(no: f64) -> String {
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
	quantity
}

fn fetch_price_map(portfolio: &Portfolio) -> Result<HashMap<AssetTag, f64>, Box<dyn Error>> {
	let portfolio_assets = portfolio.funded_symbols().into_iter().collect::<Vec<_>>();
	let mut asset_price = if portfolio_assets.is_empty() {
		HashMap::new()
	} else {
		let symbol_asset = portfolio_assets
			.iter()
			.map(|it| (it.as_str().to_string(), it.clone())).collect::<HashMap<String, _>>();
		let symbols = symbol_asset.keys().cloned().collect::<Vec<_>>();
		let symbol_price = smarket::yf::price_assets(&symbols)?
			.iter()
			.map(|(symbol, result)| {
				let usd_price = match result {
					PricingResult::Priced { usd_price, .. } => *usd_price,
					_ => panic!("missing price")
				};
				(symbol.to_string(), usd_price.as_f64())
			})
			.collect::<HashMap<String, _>>();
		symbol_price.into_iter().map(|(symbol, price)| {
			let asset_tag = symbol_asset.get(&symbol).expect("asset-tag").to_owned();
			(asset_tag, price)
		}).collect::<HashMap<AssetTag, _>>()
	};
	asset_price.insert(AssetTag::Usd, 1.0);
	Ok(asset_price)
}