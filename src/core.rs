#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Ramp { Golden, Flat }

impl Ramp {
	pub fn as_f64(&self) -> f64 {
		match self {
			Ramp::Golden => 1.618f64,
			Ramp::Flat => 1.0,
		}
	}

	pub fn pow_weight(&self, pos: usize) -> f64 { self.as_f64().powf(pos as f64) }
}

