extern crate clap;
extern crate hex;
extern crate smarket;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::fmt::Display;

use smarket::yf::PricingResult;

fn main() -> Result<(), Box<dyn Error>> {
	let lots: Vec<Lot> = vec![Lot {
		uid: 1,
		share_count: ShareCount(100.0),
		asset_type: AssetType::Usx("TSLA".to_string()),
		custodian: Custodian("robinhood".to_string()),
	}];
	let ladder = Ladder {
		assets: vec![
			AssetType::Usx("SPCE".into()),
			AssetType::Usx("GBTC".into()),
			AssetType::Usx("PEP".into()),
			AssetType::Usx("TSLA".into())
		]
	};

	let yaml = clap::load_yaml!("cli.yaml");
	let matches = clap::App::from(yaml).get_matches();
	if let Some(_) = matches.subcommand_matches("status") {
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
	} else {
		eprintln!("No command found");
	}
	Ok(())
}

fn counts(lots: &Vec<Lot>) -> HashMap<String, f64> {
	let mut map: HashMap<String, f64> = HashMap::new();
	for lot in lots {
		let AssetType::Usx(ref symbol) = lot.asset_type;
		let previous = map.get(symbol).cloned().unwrap_or(0.0);
		let next = previous + lot.share_count.as_f64();
		map.insert(symbol.to_string(), next);
	}
	map
}

#[derive(Debug)]
struct Holding {
	pub symbol: String,
	pub lots: Vec<Lot>,
}

#[derive(Debug)]
struct Ladder {
	pub assets: Vec<AssetType>
}

impl Ladder {
	pub fn ordered_symbols(&self) -> Vec<String> {
		self.assets.iter().map(|it| {
			let AssetType::Usx(ref symbol) = it;
			symbol.to_uppercase()
		}).collect()
	}
	pub fn weights(&self) -> HashMap<String, f64> {
		self.assets
			.iter()
			.enumerate()
			.map(|(i, asset_type)| {
				let AssetType::Usx(ref symbol) = asset_type;
				(symbol.to_uppercase(), 1.618f64.powf(i as f64))
			})
			.collect::<HashMap<String, _>>()
	}
	pub fn portions(&self) -> HashMap<String, f64> {
		let weights = self.weights();
		let full_weight: f64 = weights.values().sum();
		weights.iter()
			.map(|(symbol, weight)| (symbol.to_string(), *weight / full_weight))
			.collect::<HashMap<String, _>>()
	}
}

#[derive(Clone, Debug)]
pub enum AssetType { Usx(String) }

impl AssetType {
	pub fn name(&self) -> &str {
		match self {
			AssetType::Usx(name) => name
		}
	}
}

#[derive(Clone, Debug)]
pub struct Lot {
	pub uid: u64,
	pub share_count: ShareCount,
	pub asset_type: AssetType,
	pub custodian: Custodian,
}

impl Lot {
	pub fn uid_pretty(&self) -> String {
		hex::encode(self.uid.to_be_bytes())
	}
}


#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
pub struct ShareCount(f64);

impl ShareCount {
	pub fn as_f64(&self) -> f64 { self.0 }
}

impl Display for ShareCount {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_fmt(format_args!("{}", self.0))
	}
}

#[derive(Clone, Debug)]
pub struct Custodian(String);

impl Custodian {
	pub fn name(&self) -> &str {
		&self.0
	}
}
