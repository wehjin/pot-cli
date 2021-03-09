use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::asset_tag::AssetTag;
use crate::core::Ramp;
use crate::disk;
use crate::ladder::Ladder;
use crate::lot::Lot;
use crate::portfolio::Portfolio;

pub trait Pot: Clone {
	fn is_not_initialized(&self) -> bool;
	fn init_if_not(&self) -> Result<(), Box<dyn Error>>;
	fn init(&self) -> Result<(), Box<dyn Error>>;
	fn subpot(&self, name: &str) -> Box<Self>;

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

	fn read_ladder(&self) -> Result<Ladder, Box<dyn Error>>;

	fn read_lot_assets(&self) -> Result<HashSet<AssetTag>, Box<dyn Error>>;
	fn read_deep_lot_assets(&self) -> Result<HashSet<AssetTag>, Box<dyn Error>>;

	fn read_deep_subpots(&self) -> Result<Vec<(AssetTag, Box<Self>)>, Box<dyn Error>> {
		let local_subpots = self.read_lot_assets()?
			.into_iter()
			.filter(AssetTag::is_subpot)
			.map(|asset| {
				let pot = self.subpot(asset.as_folder_name());
				(asset, pot)
			})
			.collect::<Vec<_>>();
		let mut subpots = Vec::new();
		subpots.extend(local_subpots.clone());
		for (_, pot) in local_subpots {
			let deep_subpots = pot.read_deep_subpots()?;
			subpots.extend(deep_subpots);
		}
		Ok(subpots)
	}

	fn read_market_value(&self, prices: &HashMap<AssetTag, f64>) -> Result<f64, Box<dyn Error>> {
		let portfolio = Portfolio {
			lots: self.read_lots()?,
			free_cash: self.read_cash()?,
		};
		let value = portfolio.market_value(prices);
		Ok(value)
	}
	fn read_market_values(&self, prices: &HashMap<AssetTag, f64>) -> Result<HashMap<AssetTag, f64>, Box<dyn Error>> {
		let portfolio = Portfolio {
			lots: self.read_lots()?,
			free_cash: self.read_cash()?,
		};
		let value = portfolio.market_values(prices);
		Ok(value)
	}
}

impl Pot for FolderPot {
	fn is_not_initialized(&self) -> bool { csv::Reader::from_path(self.lots_file()).is_err() }
	fn init_if_not(&self) -> Result<(), Box<dyn Error>> {
		std::fs::create_dir_all(self.path.as_path())?;
		if self.is_not_initialized() {
			self.init()
		} else {
			Ok(())
		}
	}
	fn init(&self) -> Result<(), Box<dyn Error>> {
		self.write_lots(&Vec::new())?;
		self.write_cash(0.0)?;
		self.write_ramp(Ramp::Golden)?;
		Ok(())
	}

	fn subpot(&self, name: &str) -> Box<Self> {
		let sub_path = self.path.join(name);
		Box::new(FolderPot { path: sub_path })
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
				.filter(|s| !s.trim().is_empty())
				.map(|s| AssetTag::from(s.trim()))
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

	fn read_ladder(&self) -> Result<Ladder, Box<dyn Error>> {
		let ladder = Ladder { targets: self.read_targets()?, ramp: self.read_ramp()? };
		Ok(ladder)
	}

	fn read_lot_assets(&self) -> Result<HashSet<AssetTag>, Box<dyn Error>> {
		let set = self.read_lots()?
			.iter()
			.filter(|lot| lot.share_count.as_f64() > 0.0)
			.map(|lot| &lot.asset_tag)
			.cloned()
			.collect::<HashSet<_>>();
		Ok(set)
	}

	fn read_deep_lot_assets(&self) -> Result<HashSet<AssetTag>, Box<dyn Error>> {
		let top_assets = self.read_lot_assets()?;
		let (sub_tags, top_tags): (Vec<AssetTag>, Vec<AssetTag>) = top_assets.into_iter().partition(AssetTag::is_subpot);
		let mut deep_assets = HashSet::new();
		deep_assets.extend(top_tags);
		for tag in sub_tags {
			let subpot = self.subpot(tag.as_folder_name());
			let subpot_assets = subpot.read_deep_lot_assets()?;
			deep_assets.extend(subpot_assets);
		}
		Ok(deep_assets)
	}
}

#[derive(Clone, Debug)]
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
