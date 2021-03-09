use crate::asset_tag::AssetTag;

pub fn title(s: &str) {
	println!("{}", s);
	let width = s.chars().as_str().len();
	println!("{:=<1$}", "", width);
}

pub fn targets(targets: &Vec<AssetTag>) {
	let symbols = targets.iter().map(|it| it.as_str().to_string()).collect::<Vec<_>>();
	let line: String = symbols.join(",");
	println!("{}", line);
}