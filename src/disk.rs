use std::error::Error;

use crate::{AssetType, Custodian, Lot, ShareCount};

pub fn read_lots() -> Result<Vec<Lot>, Box<dyn Error>> {
	let lots: Vec<Lot> = vec![Lot {
		uid: 1,
		share_count: ShareCount(100.0),
		asset_type: AssetType::Usx("TSLA".to_string()),
		custodian: Custodian("robinhood".to_string()),
	}];
	Ok(lots)
}

pub fn init() {}
