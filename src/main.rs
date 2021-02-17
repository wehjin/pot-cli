extern crate clap;
extern crate csv;
extern crate hex;
extern crate rand;
extern crate serde;
extern crate smarket;

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub use lot::*;

mod lot;
mod disk;
mod cli;

fn main() -> Result<(), Box<dyn Error>> {
	let ladder = Ladder {
		assets: vec![
			AssetTag("SPCE".into()),
			AssetTag("GBTC".into()),
			AssetTag("PEP".into()),
			AssetTag("TSLA".into())
		]
	};

	let yaml = clap::load_yaml!("cli.yaml");
	let matches = clap::App::from(yaml).get_matches();
	if let Some(_) = matches.subcommand_matches("lots") {
		cli::lots()?;
	} else if let Some(_) = matches.subcommand_matches("init") {
		cli::init()?;
	} else if let Some(_) = matches.subcommand_matches("status") {
		cli::status(ladder)?;
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
	pub assets: Vec<AssetTag>
}

impl Ladder {
	pub fn ordered_symbols(&self) -> Vec<String> {
		self.assets.iter().map(|it| {
			it.as_str().to_uppercase()
		}).collect()
	}
	pub fn weights(&self) -> HashMap<String, f64> {
		self.assets
			.iter()
			.enumerate()
			.map(|(i, asset_type)| {
				let symbol = asset_type.as_str();
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetTag(String);

impl AssetTag {
	pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ShareCount(f64);

impl ShareCount {
	pub fn as_f64(&self) -> f64 { self.0 }
}

impl Display for ShareCount {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_fmt(format_args!("{}", self.0))
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Custodian(String);

impl Custodian {
	pub fn as_str(&self) -> &str {
		&self.0
	}
}
