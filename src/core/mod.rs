pub use deep_asset::*;
pub use pot_path::*;

mod pot_path;
mod deep_asset;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Ramp { Golden, Flat }

impl Ramp {
	pub fn from_str(s: &str) -> Self {
		let ramp = match s.to_lowercase().trim() {
			"golden" => Ramp::Golden,
			"flat" => Ramp::Flat,
			_ => Ramp::Golden
		};
		ramp
	}
	pub fn as_f64(&self) -> f64 {
		match self {
			Ramp::Golden => 1.618f64,
			Ramp::Flat => 1.0,
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Ramp::Golden => "golden",
			Ramp::Flat => "flat",
		}
	}

	pub fn pow_weight(&self, pos: usize) -> f64 { self.as_f64().powf(pos as f64) }
}

