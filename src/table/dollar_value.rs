use crate::table::Column;

pub struct DollarValueColumn {
	rows: Vec<f64>,
}

impl DollarValueColumn {
	pub fn new(rows: &Vec<f64>) -> Self {
		DollarValueColumn { rows: rows.to_owned() }
	}
}

impl Column for DollarValueColumn {
	fn rows(&self) -> usize { self.rows.len() }
	fn printout(&self, row: usize) -> String {
		let text = shorten_dollars(self.rows.get(row).cloned().unwrap_or(0.0));
		format!("{:>6}", text)
	}
}

pub fn shorten_dollars(no: f64) -> String {
	if no.is_nan() {
		"$NAN".to_string()
	} else if no == 0.0 {
		"$0".to_string()
	} else if no.is_sign_negative() {
		format!("(${})", shorten_abs(no))
	} else {
		format!("${}", shorten_abs(no))
	}
}

pub fn shorten_abs(no: f64) -> String {
	let pos = no.abs();
	let quantity = if pos >= 1e12 {
		"1.0T+".to_string()
	} else {
		let (short_pos, unit) = if pos >= 1e9 {
			(pos / 1e9, "B")
		} else if pos >= 1e6 {
			(pos / 1e6, "M")
		} else if pos >= 1e3 {
			(pos / 1e3, "K")
		} else {
			(pos, "")
		};
		let s = format!("{:07.3}", short_pos);
		let digits = if short_pos >= 100.0 {
			&s[..3]
		} else if short_pos >= 10.0 {
			&s[1..5]
		} else if short_pos >= 1.0 {
			&s[2..6]
		} else {
			&s[3..]
		};
		format!("{}{}", digits, unit)
	};
	quantity
}