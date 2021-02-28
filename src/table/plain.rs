use crate::table::Column;

pub struct PlainColumn {
	rows: Vec<String>,
	max_row_width: usize,
}

impl Column for PlainColumn {
	fn rows(&self) -> usize { self.rows.len() }
	fn printout(&self, row: usize) -> String {
		let s = match self.rows.get(row) {
			None => "",
			Some(s) => s.as_str()
		};
		format!("{:1$}", s, self.max_row_width)
	}
}

impl<S: ToString> From<&Vec<S>> for PlainColumn {
	fn from(rows: &Vec<S>) -> Self {
		let rows = rows.iter().map(ToString::to_string).collect::<Vec<_>>();
		let max_row_width = rows.iter().map(|it| it.chars().count()).max().unwrap_or(0);
		PlainColumn { rows, max_row_width }
	}
}
