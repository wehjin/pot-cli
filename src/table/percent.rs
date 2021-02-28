use crate::table::Column;

pub struct PercentColumn {
	rows: Vec<f64>,
}

impl PercentColumn {
	pub fn new(rows: &Vec<f64>) -> Self {
		PercentColumn { rows: rows.to_owned() }
	}
}

impl Column for PercentColumn {
	fn rows(&self) -> usize { self.rows.len() }
	fn printout(&self, row: usize) -> String {
		let percent = self.rows.get(row).cloned().unwrap_or(0.0) * 100.0;
		format!("{:>6.2}%", percent)
	}
}
