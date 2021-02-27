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

use lot::*;

use crate::asset_tag::AssetTag;

mod asset_tag;
mod cli;
mod core;
mod disk;
mod ladder;
mod lot;
mod portfolio;
mod pot;
mod print;

fn main() -> Result<(), Box<dyn Error>> {
	let yaml = clap::load_yaml!("cli.yaml");
	let matches = clap::App::from(yaml).get_matches();
	if let Some(_) = matches.subcommand_matches("init") {
		cli::init()?;
	} else if let Some(_) = matches.subcommand_matches("status") {
		cli::status()?;
	} else if let Some(matches) = matches.subcommand_matches("value") {
		let verbose = matches.is_present("verbose");
		cli::value(verbose)?;
	} else if let Some(_) = matches.subcommand_matches("lots") {
		cli::lots()?;
	} else if let Some(matches) = matches.subcommand_matches("assets") {
		let go_deep = matches.is_present("deep");
		cli::assets(go_deep)?;
	} else if let Some(_) = matches.subcommand_matches("cash") {
		cli::cash()?;
	} else if let Some(ramp_matches) = matches.subcommand_matches("ramp") {
		if let Some(ramp_set_matches) = ramp_matches.subcommand_matches("set") {
			let s = ramp_set_matches.value_of("RAMP").expect("ramp").to_lowercase();
			cli::set_ramp(&s)?;
		} else {
			cli::ramp()?;
		}
	} else if let Some(targets_matches) = matches.subcommand_matches("targets") {
		if let Some(add_matches) = targets_matches.subcommand_matches("add") {
			let symbols = add_matches.value_of("SYMBOLS").expect("symbols");
			cli::add_targets(symbols)?;
		} else {
			cli::targets()?;
		}
	} else if let Some(matches) = matches.subcommand_matches("shares") {
		// TODO Make this a subcommand of lots.
		let custodian = matches.value_of("CUSTODIAN").expect("custodian");
		let symbol = matches.value_of("SYMBOL").expect("symbol").to_uppercase();
		let count = matches.value_of("COUNT").map(|s| s.parse::<f64>().expect("count"));
		cli::shares(&custodian, &symbol, count)?;
	} else if let Some(matches) = matches.subcommand_matches("set") {
		if let Some(matches) = matches.subcommand_matches("cash") {
			let value = matches.value_of("VALUE").expect("value").parse::<f64>()?;
			cli::set_cash(value)?;
		} else {
			println!("Set what?");
		}
	} else if let Some(matches) = matches.subcommand_matches("add") {
		if let Some(matches) = matches.subcommand_matches("lot") {
			let custody = matches.value_of("CUSTODY").expect("custody");
			let symbol = matches.value_of("SYMBOL").expect("symbol");
			let asset = AssetTag::from(symbol);
			let share_count = matches.value_of("SHARECOUNT").expect("sharecount").parse::<f64>()?;
			let uid = matches.value_of("UID").map_or(Ok(None), |it| it.parse::<u64>().map(Some))?;
			cli::add_lot(custody, &asset, share_count, uid)?;
		} else if let Some(matches) = matches.subcommand_matches("target") {
			let symbol = matches.value_of("SYMBOL").expect("symbol");
			cli::add_targets(symbol)?;
		} else {
			println!("Add what?");
		}
	} else {
		cli::status()?;
	}
	Ok(())
}

#[derive(Debug)]
pub struct Holding {
	pub symbol: String,
	pub lots: Vec<Lot>,
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
