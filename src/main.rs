extern crate clap;
extern crate hex;
extern crate smarket;

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Display;

mod disk;
mod cli;

fn main() -> Result<(), Box<dyn Error>> {

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
	if let Some(_) = matches.subcommand_matches("lots") {
		cli::lots()?;
	} else if let Some(_) = matches.subcommand_matches("init") {
		cli::init()?;
	} else if let Some(_) = matches.subcommand_matches("status") {
		cli::status( ladder)?;
	} else {
		eprintln!("No command found");
	}
	Ok(())
}


#[derive(Debug)]
pub struct Holding {
	pub symbol: String,
	pub lots: Vec<Lot>,
}

#[derive(Debug)]
pub struct Ladder {
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
