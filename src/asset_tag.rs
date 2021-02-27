use std::fmt;
use std::fmt::Formatter;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;

#[derive(Hash, Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
pub enum AssetTag {
	Equity(String),
	Pot(String),
	Usd,
}

impl AssetTag {
	pub fn is_subpot(&self) -> bool {
		match self {
			AssetTag::Pot(_) => true,
			_ => false,
		}
	}
	pub fn is_equity(&self) -> bool {
		match self {
			AssetTag::Equity(_) => true,
			_ => false,
		}
	}
	pub fn as_str(&self) -> &str {
		match self {
			AssetTag::Equity(s) => s.as_str(),
			AssetTag::Pot(s) => s.as_str(),
			AssetTag::Usd => "USD",
		}
	}
	pub fn as_folder_name(&self) -> &str {
		match self {
			AssetTag::Pot(s) => &s[1..],
			_ => panic!("Wrong variant")
		}
	}
}

impl ToString for AssetTag {
	fn to_string(&self) -> String { self.as_str().to_string() }
}

impl<T: AsRef<str>> From<T> for AssetTag {
	fn from(t: T) -> Self {
		let s = t.as_ref();
		if s.starts_with(":") {
			AssetTag::Pot(format!(":{}", s[1..].to_lowercase()))
		} else {
			let symbol = s.to_uppercase();
			if symbol == "USD" {
				AssetTag::Usd
			} else {
				AssetTag::Equity(symbol)
			}
		}
	}
}


impl<'de> Deserialize<'de> for AssetTag {
	fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
		where D: Deserializer<'de> {
		deserializer.deserialize_any(AssetTagVisitor)
	}
}

struct AssetTagVisitor;

impl<'de> Visitor<'de> for AssetTagVisitor {
	type Value = AssetTag;

	fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
		formatter.write_str("an AssetTag string")
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
		where E: de::Error, {
		let asset_tag = AssetTag::from(v);
		Ok(asset_tag)
	}
}

impl Serialize for AssetTag {
	fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
		S: Serializer {
		let s = self.as_str().to_string();
		serializer.serialize_str(&s)
	}
}
