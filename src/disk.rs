use std::error::Error;

use crate::Lot;

const LOTS_CSV: &str = "lots.csv";

pub fn is_not_initialized() -> bool {
	csv::Reader::from_path(LOTS_CSV).is_err()
}

pub fn init() -> Result<(), Box<dyn Error>> {
	let lots: Vec<Lot> = Vec::new();
	let mut writer = csv::Writer::from_path(LOTS_CSV)?;
	writer.serialize(lots)?;
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

