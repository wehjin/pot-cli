extern crate clap;
extern crate hex;
extern crate smarket;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
	let yaml = clap::load_yaml!("cli.yaml");
	let matches = clap::App::from(yaml).get_matches();
	if let Some(_matches) = matches.subcommand_matches("lots") {
		lots::main()
	} else {
		eprintln!("No command found");
	}
	Ok(())
}

mod lots;
