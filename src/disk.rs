use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

use crate::Lot;

const LOTS_CSV: &str = "lots.csv";
const CASH_TXT: &str = "cash.txt";

pub fn is_not_initialized() -> bool {
	csv::Reader::from_path(LOTS_CSV).is_err()
}

pub fn init() -> Result<(), Box<dyn Error>> {
	let lots: Vec<Lot> = Vec::new();
	let mut writer = csv::Writer::from_path(LOTS_CSV)?;
	writer.serialize(lots)?;
	Ok(())
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

