use std::error::Error;

use crate::Lot;

pub fn init() {}

pub fn read_lots() -> Result<Vec<Lot>, Box<dyn Error>> {
	let mut lots: Vec<Lot> = Vec::new();
	let mut rdr = csv::Reader::from_path("lots.csv")?;
	for result in rdr.deserialize() {
		let lot: Lot = result?;
		lots.insert(0, lot);
	}
	lots.reverse();
	Ok(lots)
}

