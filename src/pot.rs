use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::asset_tag::AssetTag;
use crate::core::Ramp;
use crate::disk;
use crate::lot::Lot;

pub trait Pot {
	fn is_not_initialized(&self) -> bool;
	fn init(&self) -> Result<(), Box<dyn Error>>;

	fn read_cash(&self) -> Result<f64, Box<dyn Error>>;
	fn write_cash(&self, value: f64) -> Result<(), Box<dyn Error>>;

	fn read_ramp(&self) -> Result<Ramp, Box<dyn Error>>;
	fn write_ramp(&self, ramp: Ramp) -> Result<(), Box<dyn Error>>;

	fn read_lots(&self) -> Result<Vec<Lot>, Box<dyn Error>>;
	fn write_lots(&self, lots: &Vec<Lot>) -> Result<(), Box<dyn Error>>;

	fn read_shares(&self, custodian: &str, symbol: &str) -> Result<f64, Box<dyn Error>>;
	fn write_shares(&self, custodian: &str, symbol: &str, count: f64) -> Result<u64, Box<dyn Error>>;

	fn read_targets(&self) -> Result<Vec<AssetTag>, Box<dyn Error>>;
	fn write_targets(&self, targets: &Vec<AssetTag>) -> Result<(), Box<dyn Error>>;
}

impl Pot for FolderPot {
	fn is_not_initialized(&self) -> bool { csv::Reader::from_path(self.lots_file()).is_err() }
	fn init(&self) -> Result<(), Box<dyn Error>> {
		self.write_lots(&Vec::new())?;
		self.write_cash(0.0)?;
		self.write_ramp(Ramp::Golden)?;
		Ok(())
	}
	fn read_cash(&self) -> Result<f64, Box<dyn Error>> {
		disk::read_f64(&self.cash_file())
	}

	fn write_cash(&self, value: f64) -> Result<(), Box<dyn Error>> {
		let value_s = value.to_string();
		disk::write_string(&self.cash_file(), &value_s)
	}

	fn read_ramp(&self) -> Result<Ramp, Box<dyn Error>> {
		let string = disk::read_string(&self.ramp_file()).unwrap_or("golden".to_string());
		let ramp = Ramp::from_str(&string);
		Ok(ramp)
	}

	fn write_ramp(&self, ramp: Ramp) -> Result<(), Box<dyn Error>> {
		disk::write_string(&self.ramp_file(), ramp.as_str())
	}

	fn read_lots(&self) -> Result<Vec<Lot>, Box<dyn Error>> {
		let mut lots = Vec::new();
		let mut rdr = csv::Reader::from_path(self.lots_file())?;
		for result in rdr.deserialize() {
			let lot: Lot = result?;
			lots.insert(0, lot);
		}
		lots.reverse();
		Ok(lots)
	}
	fn write_lots(&self, lots: &Vec<Lot>) -> Result<(), Box<dyn Error>> {
		let mut wtr = csv::Writer::from_path(self.lots_file())?;
		for lot in lots {
			wtr.serialize(lot)?;
		}
		wtr.flush()?;
		Ok(())
	}

	fn read_shares(&self, custodian: &str, symbol: &str) -> Result<f64, Box<dyn Error>> {
		let tag = AssetTag::from(symbol);
		let lots = self.read_lots()?;
		let lot = lots.into_iter().find(|lot| lot.has_tag(&tag) && lot.has_custodian(custodian));
		let count = if let Some(lot) = lot {
			lot.share_count.as_f64()
		} else {
			0.0
		};
		Ok(count)
	}

	fn write_shares(&self, custodian: &str, symbol: &str, count: f64) -> Result<u64, Box<dyn Error>> {
		let tag = AssetTag::from(symbol);
		let mut lot_id: Option<u64> = None;
		let new_lots = self.read_lots()?.into_iter().map(|lot| {
			if lot.has_tag(&tag) && lot.has_custodian(custodian) {
				lot_id = Some(lot.uid);
				lot.with_share_count(count)
			} else {
				lot
			}
		}).collect::<Vec<_>>();
		self.write_lots(&new_lots)?;
		Ok(lot_id.expect("lot it"))
	}
	fn read_targets(&self) -> Result<Vec<AssetTag>, Box<dyn Error>> {
		let mut file_s = String::new();
		let file_open = File::open(self.team_file());
		if file_open.is_err() {
			Ok(Vec::new())
		} else {
			file_open?.read_to_string(&mut file_s)?;
			let asset_tags = file_s
				.split("\n")
				.into_iter()
				.map(|s| AssetTag::from(s))
				.collect::<Vec<_>>();
			Ok(asset_tags)
		}
	}

	fn write_targets(&self, targets: &Vec<AssetTag>) -> Result<(), Box<dyn Error>> {
		let symbols = targets.iter().map(|tag| tag.as_str().to_string()).collect::<Vec<String>>();
		let targets: String = symbols.join("\n");
		let mut file = File::create(self.team_file())?;
		file.write_all(targets.as_bytes())?;
		Ok(())
	}
}

pub struct FolderPot { path: PathBuf }

impl FolderPot {
	pub fn new() -> Self { FolderPot { path: PathBuf::from(".") } }
	pub fn path(&self) -> &Path { &self.path }
	fn file_path(&self, filename: &str) -> PathBuf { self.path.join(filename) }
	fn cash_file(&self) -> PathBuf { self.file_path("cash.txt") }
	fn ramp_file(&self) -> PathBuf { self.file_path("ramp.txt") }
	fn lots_file(&self) -> PathBuf { self.file_path("lots.csv") }
	fn team_file(&self) -> PathBuf { self.file_path("team.txt") }
}
