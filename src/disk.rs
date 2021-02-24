use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

use crate::{AssetTag, Lot};
use crate::core::Ramp;
use crate::ladder::Ladder;
use crate::portfolio::Portfolio;

const LOTS_CSV: &str = "lots.csv";
const CASH_TXT: &str = "cash.txt";
const TEAM_TXT: &str = "team.txt";
const RAMP_TXT: &str = "ramp.txt";

pub fn is_not_initialized() -> bool {
	csv::Reader::from_path(LOTS_CSV).is_err()
}

pub fn init() -> Result<(), Box<dyn Error>> {
	let lots: Vec<Lot> = Vec::new();
	let mut writer = csv::Writer::from_path(LOTS_CSV)?;
	writer.serialize(lots)?;
	Ok(())
}

pub fn read_portfolio() -> Result<Portfolio, Box<dyn Error>> {
	let portfolio = Portfolio {
		lots: read_lots()?,
		free_cash: read_cash()?,
	};
	Ok(portfolio)
}

pub fn read_ladder() -> Result<Ladder, Box<dyn Error>> {
	let targets = read_targets()?.into_iter().map(AssetTag).collect::<Vec<_>>();
	let ramp = read_ramp()?;
	Ok(Ladder { targets, ramp })
}

pub fn read_ramp() -> Result<Ramp, Box<dyn Error>> {
	let string = read_string(RAMP_TXT).unwrap_or("golden".to_string());
	let ramp = Ramp::from_str(&string);
	Ok(ramp)
}

pub fn write_ramp(ramp: Ramp) -> Result<(), Box<dyn Error>> {
	write_string(RAMP_TXT, ramp.as_str())
}

pub fn read_targets() -> Result<Vec<String>, Box<dyn Error>> {
	let mut file_s = String::new();
	let file_open = File::open(TEAM_TXT);
	if file_open.is_err() {
		Ok(Vec::new())
	} else {
		file_open?.read_to_string(&mut file_s)?;
		let symbols = file_s
			.split("\n")
			.into_iter()
			.map(|s| {
				let s = s.trim();
				s.to_uppercase()
			})
			.filter(|s| !s.is_empty())
			.collect::<Vec<_>>();
		Ok(symbols)
	}
}

pub fn write_targets(targets: &Vec<String>) -> Result<(), Box<dyn Error>> {
	let targets: String = targets.join("\n");
	let mut file = File::create(TEAM_TXT)?;
	file.write_all(targets.as_bytes())?;
	Ok(())
}

pub fn read_shares(custodian: &str, symbol: &str) -> Result<f64, Box<dyn Error>> {
	let lots = read_lots()?;
	let lot = lots.into_iter().find(|lot| lot.has_symbol(symbol) && lot.has_custodian(custodian));
	let count = if let Some(lot) = lot {
		lot.share_count.as_f64()
	} else {
		0.0
	};
	Ok(count)
}

pub fn write_shares(custodian: &str, symbol: &str, count: f64) -> Result<u64, Box<dyn Error>> {
	let mut lot_id: Option<u64> = None;
	let new_lots = read_lots()?.into_iter().map(|lot| {
		if lot.has_symbol(symbol) && lot.has_custodian(custodian) {
			lot_id = Some(lot.uid);
			lot.with_share_count(count)
		} else {
			lot
		}
	}).collect::<Vec<_>>();
	write_lots(&new_lots)?;
	Ok(lot_id.expect("lot it"))
}

pub fn read_cash() -> Result<f64, Box<dyn Error>> {
	read_f64(CASH_TXT)
}

pub fn write_cash(value: f64) -> Result<(), Box<dyn Error>> {
	let value_s = value.to_string();
	write_string(CASH_TXT, &value_s)
}

pub fn read_lots() -> Result<Vec<Lot>, Box<dyn Error>> {
	let mut lots: Vec<Lot> = Vec::new();
	let mut rdr = csv::Reader::from_path(LOTS_CSV)?;
	for result in rdr.deserialize() {
		let lot: Lot = result?;
		lots.insert(0, lot);
	}
	lots.reverse();
	Ok(lots)
}

pub fn write_lots(lots: &Vec<Lot>) -> Result<(), Box<dyn Error>> {
	let mut wtr = csv::Writer::from_path(LOTS_CSV)?;
	for lot in lots {
		wtr.serialize(lot)?;
	}
	wtr.flush()?;
	Ok(())
}

fn read_f64(path: &str) -> Result<f64, Box<dyn Error>> {
	let cash = read_string(path)?.parse::<f64>()?;
	Ok(cash)
}

fn read_string(path: &str) -> Result<String, Box<dyn Error>> {
	let mut s = String::new();
	File::open(path)?.read_to_string(&mut s)?;
	Ok(s)
}

fn write_string(path: &str, string: &str) -> Result<(), Box<dyn Error>> {
	File::create(path)?.write_all(string.as_bytes())?;
	Ok(())
}
