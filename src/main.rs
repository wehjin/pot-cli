extern crate clap;
extern crate hex;
extern crate smarket;

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Display;

fn main() -> Result<(), Box<dyn Error>> {
	let yaml = clap::load_yaml!("cli.yaml");
	let matches = clap::App::from(yaml).get_matches();
	if let Some(_matches) = matches.subcommand_matches("lots") {
		let lots: Vec<Lot> = vec![Lot {
			uid: 1,
			share_count: ShareCount(100.0),
			asset_type: AssetType::Usx("tsla".to_string()),
			custodian: Custodian("robinhood".to_string()),
		}];
		if lots.is_empty() {
			println!("No lots yet");
		} else {
			println!("{:16}  {:15}  {:10}  {}", "LOT ID", "CUSTODIAN", "ASSET", "SHARES");
			for ref lot in lots {
				println!("{:16}  {:15}  {:10}  {}", lot.uid_pretty(), lot.custodian.name(), lot.asset_type.name(), lot.share_count)
			}
		}
	} else if let Some(_) = matches.subcommand_matches("assets") {
		let ladder = Ladder {
			assets: vec![
				AssetType::Usx("SPCE".into()),
				AssetType::Usx("GBTC".into()),
				AssetType::Usx("PEP".into()),
				AssetType::Usx("TSLA".into())
			]
		};
		let rungs = ladder.assets
			.iter()
			.enumerate()
			.map(|(i, asset_type)| {
				let AssetType::Usx(ref symbol) = asset_type;
				Rung {
					symbol: symbol.to_owned(),
					weight: 1.618f64.powf(i as f64),
				}
			})
			.collect::<Vec<_>>();
		let full_weight: f64 = rungs.iter().map(|it| it.weight).sum();
		let portions = rungs.iter().map(|rung| (rung.symbol.to_owned(), rung.weight / full_weight)).collect::<HashMap<String, _>>();
		println!("{:10}  {:7}", "ASSET ID", "PORTION");
		for rung in rungs {
			let portion = portions.get(&rung.symbol).expect("value");
			println!("{:10}  {:<5.2}%", rung.symbol, portion * 100.0)
		}
	} else {
		eprintln!("No command found");
	}
	Ok(())
}

#[derive(Debug)]
struct Holding {
	pub symbol: String,
	pub lots: Vec<Lot>,
}

#[derive(Debug)]
struct Rung {
	pub symbol: String,
	pub weight: f64,
}

#[derive(Debug)]
struct Ladder {
	pub assets: Vec<AssetType>
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
