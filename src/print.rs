pub fn title(s: &str) {
	println!("{}", s);
	let width = s.chars().as_str().len();
	println!("{:=<1$}", "", width);
}
