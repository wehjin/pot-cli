use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use crate::asset_tag::AssetTag;
use crate::ladder::Ladder;
use crate::portfolio::Portfolio;
use crate::pot::{FolderPot, Pot};

pub fn read_portfolio() -> Result<Portfolio, Box<dyn Error>> {
	let pot = FolderPot::new();
	let portfolio = Portfolio {
		lots: pot.read_lots()?,
		free_cash: pot.read_cash()?,
	};
	Ok(portfolio)
}

pub fn read_ladder(pot: &FolderPot) -> Result<Ladder, Box<dyn Error>> {
	let targets = pot.read_targets()?.into_iter().map(AssetTag::from).collect::<Vec<_>>();
	let ramp = pot.read_ramp()?;
	Ok(Ladder { targets, ramp })
}

pub fn read_f64(path: &Path) -> Result<f64, Box<dyn Error>> {
	let cash = read_string(path)?.parse::<f64>()?;
	Ok(cash)
}

pub fn read_string(path: &Path) -> Result<String, Box<dyn Error>> {
	let mut s = String::new();
	File::open(path)?.read_to_string(&mut s)?;
	Ok(s)
}

pub fn write_string(path: &Path, string: &str) -> Result<(), Box<dyn Error>> {
	File::create(path)?.write_all(string.as_bytes())?;
	Ok(())
}
