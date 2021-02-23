use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{AssetTag, Custodian, ShareCount};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lot {
	#[serde(rename = "custody")]
	pub custodian: Custodian,
	#[serde(rename = "symbol")]
	pub asset_tag: AssetTag,
	#[serde(rename = "count")]
	pub share_count: ShareCount,
	#[serde(default = "Lot::random_uid")]
	pub uid: u64,
}

impl Lot {
	pub fn random_uid() -> u64 {
		rand::thread_rng().gen()
	}
	pub fn uid_pretty(&self) -> String {
		hex::encode(self.uid.to_be_bytes())
	}
	pub fn symbol_string(&self) -> String { self.asset_tag.as_str().to_string() }
	pub fn is_funded(&self) -> bool { self.share_count.is_non_zero() }
	pub fn has_symbol(&self, symbol: &str) -> bool { symbol == self.asset_tag.as_str() }
	pub fn has_custodian(&self, custodian: &str) -> bool { custodian == self.custodian.as_str() }

	pub fn with_share_count(&self, count: f64) -> Self {
		Lot { share_count: ShareCount(count), ..self.clone() }
	}
}
