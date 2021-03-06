use std::collections::{HashMap, HashSet};
use std::error::Error;

use smarket::yf::PricingResult;

use table::plain::PlainColumn;

use crate::{Custodian, disk, Lot, print, ShareCount, table};
use crate::asset_tag::AssetTag;
use crate::core::{AssetGroup, DeepAsset, into_groups, PotPath, Ramp};
use crate::pot::{FolderPot, Pot};
use crate::table::dollar_value::{DollarValueColumn, shorten_abs, shorten_dollars};
use crate::table::percent::PercentColumn;
use crate::table::Table;

pub fn init() -> Result<(), Box<dyn Error>> {
	let mut pot = FolderPot::new();
	if pot.is_not_initialized() {
		pot.init()?;
		println!("Initialized pot in {}", pot.path().display());
	} else {
		println!("Skipped reinitializing existing pot in {}", pot.path().display());
	}
	Ok(())
}

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
	let ladder = pot.read_ladder()?;
	let (symbols, portions) = {
		let mut asset_portions = ladder.asset_portions();
		asset_portions.reverse();
		let symbols = asset_portions.iter().map(|(asset, _)| asset.to_string()).collect();
		let portions = asset_portions.iter().map(|(_, portion)| portion.to_owned()).collect();
		(symbols, portions)
	};
	let table = Table::new(vec![
		Box::new(PlainColumn::from(&symbols)),
		Box::new(PercentColumn::new(&portions))
	]);
	for i in 0..table.lines() {
		println!("{}", table.printout(i));
	}
	Ok(())
}

pub fn add_targets(symbols: &str) -> Result<(), Box<dyn Error>> {
	let asset_tags = symbols
		.split(",")
		.map(|s| AssetTag::from(s.trim()))
		.collect::<Vec<_>>();
	let pot = FolderPot::new();
	let mut targets = pot.read_targets()?;
	let original = targets.len();
	asset_tags.iter().rev().for_each(|tag| {
		let position = targets.iter().position(|t| t == tag);
		if position.is_none() {
			targets.insert(0, tag.clone());
		}
	});
	if targets.len() > original {
		pot.write_targets(&targets)?;
	}
	print::targets(&targets);
	Ok(())
}

pub fn remove_targets(symbols: &str) -> Result<(), Box<dyn Error>> {
	let asset_tags = symbols
		.split(",")
		.map(|s| AssetTag::from(s.trim()))
		.collect::<Vec<_>>();
	let pot = FolderPot::new();
	let mut targets = pot.read_targets()?;
	let original = targets.len();
	asset_tags.iter().for_each(|tag| {
		let position = targets.iter().position(|t| t == tag);
		if let Some(index) = position {
			targets.remove(index);
		}
	});
	if targets.len() < original {
		pot.write_targets(&targets)?;
	}
	print::targets(&targets);
	Ok(())
}

pub fn promote_target(symbol: &str) -> Result<(), Box<dyn Error>> {
	let asset = AssetTag::from(symbol);
	let pot = FolderPot::new();
	let mut ladder = pot.read_ladder()?;
	match ladder.promote_target(&asset) {
		None => {
			println!("{} is not a pot target", asset.as_str());
		}
		Some(position) => {
			pot.write_targets(&ladder.targets)?;
			println!("Promoted {} to position {}", asset.as_str(), position);
		}
	};
	Ok(())
}

pub fn demote_target(symbol: &str) -> Result<(), Box<dyn Error>> {
	let asset = AssetTag::from(symbol);
	let pot = FolderPot::new();
	let mut ladder = pot.read_ladder()?;
	match ladder.demote_target(&asset) {
		None => {
			println!("{} is not a pot target", asset.as_str());
		}
		Some(position) => {
			pot.write_targets(&ladder.targets)?;
			println!("Demoted {} to position {}", asset.as_str(), position);
		}
	};
	Ok(())
}

pub fn shares(custodian: &str, symbol: &str, count: Option<f64>) -> Result<(), Box<dyn Error>> {
	match count {
		None => {
			let pot = FolderPot::new();
			let count = pot.read_shares(&custodian, &symbol)?;
			println!("{}", count);
		}
		Some(count) => {
			let mut pot = FolderPot::new();
			let uid = pot.write_shares(&custodian, &symbol, count)?;
			println_uid(uid);
		}
	}
	Ok(())
}

pub fn add_subpot(name: &str) -> Result<(), Box<dyn Error>> {
	let pot = FolderPot::new();
	let mut sub = pot.subpot(name);
	sub.init_if_not()?;
	let lots = pot.read_lots()?;
	let tag = AssetTag::pot_from_name(name);
	let position = lots.iter().position(|lot| lot.asset_tag == tag);
	if position.is_none() {
		add_lot(tag.as_str(), &tag, 1.0, None)?;
	} else {
		print::lots(&lots);
	}
	Ok(())
}

pub fn add_lot(custody: &str, asset_tag: &AssetTag, share_count: f64, uid: Option<u64>) -> Result<(), Box<dyn Error>> {
	let mut pot = FolderPot::new();
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
		print::lots(&lots);
	}
	Ok(())
}

pub fn gather_asset(symbol: &str, dest: &PotPath) -> Result<(), Box<dyn Error>> {
	let mut dest_pot = FolderPot::from_pot_path(dest);
	let moving_tag = AssetTag::from(symbol);
	let moving_assets = FolderPot::new().read_deep_assets()?
		.into_iter()
		.filter(|asset| !asset.has_path(dest) && asset.has_tag(&moving_tag))
		.collect::<Vec<_>>();
	for ref moving_asset in moving_assets {
		let mut src_pot = FolderPot::from_pot_path(&moving_asset.pot_path);
		let src_lots = src_pot.read_lots()?;
		let (move_lots, hold_lots): (Vec<Lot>, Vec<Lot>) = src_lots
			.into_iter()
			.partition(|lot| lot.has_tag(&moving_tag));
		let fresh_lots = move_lots.iter().map(Lot::with_fresh_uid).collect::<Vec<_>>();
		dest_pot.add_lots(fresh_lots)?;
		src_pot.write_lots(&hold_lots)?;
	}
	assets()
}


fn println_uid(uid: u64) {
	println!("{:016}", uid);
}

pub fn asset_values() -> Result<(), Box<dyn Error>> {
	let mut names = Vec::new();
	let mut values = Vec::new();
	let pot = FolderPot::new();
	let prices = fetch_prices(&pot)?;
	let mut groups: Vec<AssetGroup> = into_groups(pot.read_deep_assets()?).into_iter().collect();
	groups.sort_by_key(|it| it.tag.to_owned());
	for group in groups {
		names.insert(names.len(), group.tag.to_string());
		values.insert(values.len(), group.market_value(&prices)?);
	}
	let asset_col = PlainColumn::from(&names);
	let values_col = DollarValueColumn::new(&values);
	let table = Table::new(vec![Box::new(asset_col), Box::new(values_col)]);
	for i in 0..table.lines() {
		println!("{}", table.printout(i))
	}
	Ok(())
}

pub fn value(verbose: bool) -> Result<(), Box<dyn Error>> {
	let pot = FolderPot::new();
	let prices = fetch_prices(&pot)?;
	if verbose {
		let market_values = pot.read_market_values(&prices)?;
		let mut pairs = market_values.into_iter().collect::<Vec<_>>();
		pairs.sort_by_key(|x| x.0.to_owned());
		print::title("Market Values");
		let mut total = 0.0;
		for (asset, value) in pairs {
			total += value;
			println!("{:8}  {:>8}", asset.as_str(), shorten_dollars(value));
		}
		println!("{:=<18}", "");
		println!("Total: {}", shorten_dollars(total));
	} else {
		let value = pot.read_market_value(&prices)?;
		println!("{}", shorten_dollars(value));
	}
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
	let asset_prices = fetch_prices(&pot)?;
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

pub fn assets() -> Result<(), Box<dyn Error>> {
	let pot = FolderPot::new();
	let deep_assets = pot.read_deep_assets()?;
	let mut titles = deep_assets.iter().map(DeepAsset::title).collect::<Vec<_>>();
	titles.sort();
	titles.iter().for_each(|title| {
		println!("{}", title);
	});
	Ok(())
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


fn fetch_prices(pot: &impl Pot) -> Result<HashMap<AssetTag, f64>, Box<dyn Error>> {
	let mut prices = HashMap::new();
	prices.insert(AssetTag::Usd, 1.0);
	{
		let equity_assets = pot.read_deep_lot_assets()?.into_iter().map(|it| it).collect();
		let equity_prices = fetch_equity_prices(equity_assets)?;
		prices.extend(equity_prices);
	};
	{
		let mut subpots = pot.read_deep_subpots()?;
		subpots.reverse();
		for (asset, subpot) in subpots {
			let value = subpot.read_market_value(&prices)?;
			prices.insert(asset, value);
		}
	}
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