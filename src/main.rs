extern crate clap;
extern crate hex;

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt;

pub struct Lot {
	pub uid: u64,
	pub share_count: ShareCount,
	pub asset_type: AssetType,
	pub custodian: Custodian,
}

impl Lot {
	pub fn uid_pretty(&self) -> String {
		hex::encode(self.uid.to_be_bytes())
	}
}


pub struct ShareCount(f64);

impl Display for ShareCount {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.write_fmt(format_args!("{}", self.0))
	}
}

pub enum AssetType { Usx(String) }

impl AssetType {
	pub fn name(&self) -> &str {
		match self {
			AssetType::Usx(name) => name
		}
	}
}

pub struct Custodian(String);

impl Custodian {
	pub fn name(&self) -> &str {
		&self.0
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	let yaml = clap::load_yaml!("cli.yaml");
	let matches = clap::App::from(yaml).get_matches();
	if let Some(_matches) = matches.subcommand_matches("lots") {
		let lots: Vec<Lot> = vec![Lot {
			uid: 1,
			share_count: ShareCount(100.0),
			asset_type: AssetType::Usx("tsla".to_string()),
			custodian: Custodian("robinhood".to_string()),
		}];
		if lots.is_empty() {
			println!("No lots yet");
		} else {
			println!("{:16}  {:15}  {:10}  {}", "LOT ID", "CUSTODIAN", "ASSET", "SHARES");
			for ref lot in lots {
				println!("{:16}  {:15}  {:10}  {}", lot.uid_pretty(), lot.custodian.name(), lot.asset_type.name(), lot.share_count)
			}
		}
	} else {
		eprintln!("No command found");
	}
	Ok(())
}
