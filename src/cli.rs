use std::collections::{HashMap, HashSet};
use std::error::Error;

use smarket::yf::PricingResult;

use crate::{Custodian, disk, Lot, Portfolio, print, ShareCount};
use crate::asset_tag::{AssetTag, equities_and_pots};
use crate::core::Ramp;
use crate::pot::{FolderPot, Pot};

pub fn set_cash(value: f64) -> Result<(), Box<dyn Error>> {
	let pot = FolderPot::new();
	pot.write_cash(value)
}

pub fn cash() -> Result<(), Box<dyn Error>> {
	let pot = FolderPot::new();
	let cash_value = pot.read_cash()?;
	println!("${:.2}", cash_value);
	Ok(())
}

pub fn set_ramp(ramp_s: &str) -> Result<(), Box<dyn Error>> {
	let ramp = Ramp::from_str(ramp_s);
	let pot = FolderPot::new();
	pot.write_ramp(ramp)?;
	println!("{}", ramp_s);
	Ok(())
}

pub fn ramp() -> Result<(), Box<dyn Error>> {
	let pot = FolderPot::new();
	let ramp = pot.read_ramp()?;
	println!("{}", ramp.as_str());
	Ok(())
}

pub fn targets() -> Result<(), Box<dyn Error>> {
	let pot = FolderPot::new();
	let targets = pot.read_targets()?;
	print::title("TARGETS");
	targets.iter().for_each(|tag| {
		println!("{}", tag.as_str());
	});
	Ok(())
}

pub fn add_targets(symbols: &str) -> Result<(), Box<dyn Error>> {
	let pot = FolderPot::new();
	let asset_tags = symbols
		.split(",")
		.map(|s| AssetTag::from(s.trim()))
		.collect::<Vec<_>>();
	let mut targets = pot.read_targets()?;
	let mut added = Vec::new();
	asset_tags.iter().rev().for_each(|tag| {
		let position = targets.iter().position(|t| t == tag);
		if position.is_none() {
			targets.insert(0, tag.clone());
			added.insert(0, tag.as_str().to_string());
		}
	});
	if !added.is_empty() {
		pot.write_targets(&targets)?;
	}
	println!("{}", added.join(","));
	Ok(())
}

pub fn shares(custodian: &str, symbol: &str, count: Option<f64>) -> Result<(), Box<dyn Error>> {
	let pot = FolderPot::new();
	match count {
		None => {
			let count = pot.read_shares(&custodian, &symbol)?;
			println!("{}", count);
		}
		Some(count) => {
			let uid = pot.write_shares(&custodian, &symbol, count)?;
			println_uid(uid);
		}
	}
	Ok(())
}

pub fn add_lot(custody: &str, asset_tag: &AssetTag, share_count: f64, uid: Option<u64>) -> Result<(), Box<dyn Error>> {
	let pot = FolderPot::new();
	let uid = uid.unwrap_or_else(Lot::random_uid);
	let mut lots = pot.read_lots()?;
	let existing = lots.iter().find(|it| it.uid == uid);
	if existing.is_some() {
		println!("skip: Lot {:016} already exists", uid)
	} else {
		let share_count = match asset_tag {
			AssetTag::Pot(_) => if share_count > 0.0 { 1.0 } else { 0.0 }
			_ => share_count
		};
		let lot = Lot {
			custodian: Custodian(custody.to_string()),
			asset_tag: asset_tag.to_owned(),
			share_count: ShareCount(share_count),
			uid,
		};
		lots.extend(vec![lot]);
		pot.write_lots(&lots)?;
		println_uid(uid);
	}
	Ok(())
}

fn println_uid(uid: u64) {
	println!("{:016}", uid);
}

pub fn value() -> Result<(), Box<dyn Error>> {
	let portfolio = disk::read_portfolio()?;
	let prices = fetch_prices(&portfolio)?;
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
	let asset_prices = fetch_prices(&portfolio)?;
	let mut market_values = portfolio.market_values(&asset_prices);
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
	let pot = FolderPot::new();
	println!("{:16}  {:10}  {:8}  {:8}", "LOT ID", "CUSTODY", "SYMBOL", "COUNT");
	let lots = pot.read_lots()?;
	for lot in lots {
		println!(
			"{:016x}  {:10}  {:8}  {:8}",
			lot.uid, lot.custodian.as_str(), lot.asset_tag.as_str(), lot.share_count.as_f64()
		);
	}
	Ok(())
}

pub fn lots_symbols() -> Result<(), Box<dyn Error>> {
	let pot = FolderPot::new();
	let lots = pot.read_lots()?;
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
	let pot = FolderPot::new();
	if pot.is_not_initialized() {
		pot.init()?;
		println!("Initialized pot in {}", pot.path().display());
	} else {
		println!("Skipped reinitializing existing pot in {}", pot.path().display());
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

fn fetch_prices(portfolio: &Portfolio) -> Result<HashMap<AssetTag, f64>, Box<dyn Error>> {
	let portfolio_assets = portfolio.funded_symbols().into_iter().collect::<Vec<_>>();
	let (equities, pots) = equities_and_pots(portfolio_assets);
	let equity_prices = fetch_equity_prices(equities)?;
	let subpot_prices = fetch_pot_prices(pots)?;
	let mut prices = HashMap::new();
	prices.insert(AssetTag::Usd, 1.0);
	prices.extend(equity_prices);
	prices.extend(subpot_prices);
	Ok(prices)
}

fn fetch_pot_prices(pots: Vec<AssetTag>) -> Result<HashMap<AssetTag, f64>, Box<dyn Error>> {
	let prices = pots
		.into_iter()
		.map(|pot| (pot, 42.0))
		.collect::<HashMap<AssetTag, _>>();
	Ok(prices)
}

fn fetch_equity_prices(equities: Vec<AssetTag>) -> Result<HashMap<AssetTag, f64>, Box<dyn Error>> {
	let prices_by_asset = if equities.is_empty() {
		HashMap::new()
	} else {
		let assets_by_symbol = equities
			.iter()
			.map(|it| (it.as_str().to_string(), it.clone()))
			.collect::<HashMap<String, _>>();
		let symbols = assets_by_symbol.keys().cloned().collect::<Vec<_>>();
		smarket::yf::price_assets(&symbols)?
			.iter()
			.map(|(symbol, result)| {
				let usd_price = match result {
					PricingResult::Priced { usd_price, .. } => *usd_price,
					_ => panic!("missing price")
				};
				let asset_tag = assets_by_symbol.get(symbol).expect("asset-tag").to_owned();
				(asset_tag, usd_price.as_f64())
			})
			.collect::<HashMap<AssetTag, _>>()
	};
	Ok(prices_by_asset)
}