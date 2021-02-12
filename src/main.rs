use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
	let yaml = clap::load_yaml!("cli.yaml");
	let matches = clap::App::from(yaml).get_matches();
	eprintln!("No command given");
	Ok(())
}
