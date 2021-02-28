use std::ops::Deref;

pub mod plain;
pub mod percent;

pub trait Column {
	fn rows(&self) -> usize;
	fn printout(&self, row: usize) -> String;
}

impl Column for Box<dyn Column> {
	fn rows(&self) -> usize { self.deref().rows() }
	fn printout(&self, row: usize) -> String { self.deref().printout(row) }
}

pub struct Table {
	cols: Vec<Box<dyn Column>>,
	max_col_height: usize,
}

impl Table {
	pub fn new(cols: Vec<Box<dyn Column>>) -> Self {
		let max_col_height = cols.iter().map(Column::rows).max().unwrap_or(0);
		Table { cols, max_col_height }
	}
	pub fn lines(&self) -> usize {
		self.max_col_height
	}
	pub fn printout(&self, row: usize) -> String {
		self.cols.iter().map(|it| it.printout(row)).collect::<Vec<_>>().join(COLUMN_GAP)
	}
}

const COLUMN_GAP: &str = "    ";