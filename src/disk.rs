use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

use crate::{AssetTag, Lot};
use crate::ladder::Ladder;

const LOTS_CSV: &str = "lots.csv";
const CASH_TXT: &str = "cash.txt";
const TEAM_TXT: &str = "team.txt";

pub fn is_not_initialized() -> bool {
	csv::Reader::from_path(LOTS_CSV).is_err()
}

pub fn init() -> Result<(), Box<dyn Error>> {
	let lots: Vec<Lot> = Vec::new();
	let mut writer = csv::Writer::from_path(LOTS_CSV)?;
	writer.serialize(lots)?;
	Ok(())
}

pub fn read_ladder() -> Result<Ladder, Box<dyn Error>> {
	let targets = read_targets()?.into_iter().map(AssetTag).collect::<Vec<_>>();
	Ok(Ladder { targets })
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
	let mut s = String::new();
	File::open(CASH_TXT)?.read_to_string(&mut s)?;
	let cash = s.parse::<f64>()?;
	Ok(cash)
}

pub fn write_cash(value: f64) -> Result<(), Box<dyn Error>> {
	let value_s = value.to_string();
	File::create(CASH_TXT)?.write_all(value_s.as_bytes())?;
	Ok(())
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

