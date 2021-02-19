extern crate clap;
extern crate csv;
extern crate hex;
extern crate rand;
extern crate serde;
extern crate smarket;

use std::error::Error;
use std::fmt;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use ladder::*;
use lot::*;
use portfolio::*;

mod cli;
mod disk;
mod ladder;
mod lot;
mod portfolio;

fn main() -> Result<(), Box<dyn Error>> {
	let ladder = Ladder {
		targets: vec![
			AssetTag("SPCE".into()),
			AssetTag("GBTC".into()),
			AssetTag("PEP".into()),
			AssetTag("TSLA".into())
		]
	};

	let yaml = clap::load_yaml!("cli.yaml");
	let matches = clap::App::from(yaml).get_matches();
	if let Some(_) = matches.subcommand_matches("init") {
		cli::init()?;
	} else if let Some(_) = matches.subcommand_matches("status") {
		cli::status(ladder)?;
	} else if let Some(_) = matches.subcommand_matches("lots") {
		cli::lots()?;
	} else if let Some(matches) = matches.subcommand_matches("add") {
		if let Some(matches) = matches.subcommand_matches("lot") {
			let custody = matches.value_of("CUSTODY").expect("custody");
			let symbol = matches.value_of("SYMBOL").expect("symbol").to_uppercase();
			let share_count = matches.value_of("SHARECOUNT").expect("sharecount").parse::<f64>()?;
			let uid = matches.value_of("UID").map_or(Ok(None), |it| it.parse::<u64>().map(Some))?;
			cli::add_lot(custody, &symbol, share_count, uid)?;
		} else {
			println!("Add what?");
		}
	} else {
		cli::status(ladder)?;
	}
	Ok(())
}

#[derive(Debug)]
pub struct Holding {
	pub symbol: String,
	pub lots: Vec<Lot>,
}


#[derive(Hash, Ord, PartialOrd, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct AssetTag(String);

impl AssetTag {
	pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ShareCount(f64);

impl ShareCount {
	pub fn as_f64(&self) -> f64 { self.0 }
	pub fn is_zero(&self) -> bool { self.0 == 0.0 }
	pub fn is_non_zero(&self) -> bool { !self.is_zero() }
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
