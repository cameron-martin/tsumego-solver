#[cfg(feature = "explore")]
mod explore;
#[cfg(not(feature = "explore"))]
mod explore {
    pub fn run(_filename: &str) {
        panic!("The explore command has been disabled in this compilation!");
    }
}

mod generate;

use clap::{App, AppSettings, Arg, SubCommand};
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    let matches = App::new("Tsumego Solver")
        .subcommand(
            SubCommand::with_name("explore")
                .about("Explore the game tree of a single puzzle")
                .arg(
                    Arg::with_name("file")
                        .help("The SGF file to load")
                        .short("f")
                        .long("file")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("generate")
                .about("Generate puzzles and output them as SGF files")
                .arg(
                    Arg::with_name("out")
                        .help("The directory to write the generated puzzles to")
                        .short("o")
                        .long("out")
                        .default_value("generated_puzzles")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("threads")
                        .help("The number of threads to use")
                        .long("threads")
                        .default_value("8")
                        .takes_value(true),
                ),
        )
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    match matches.subcommand() {
        ("explore", Some(matches)) => {
            let filename = matches.value_of("file").unwrap();

            explore::run(filename);

            Ok(())
        }
        ("generate", Some(matches)) => {
            let output_directory = matches.value_of("out").unwrap();
            let thread_count = matches.value_of("threads").unwrap();

            generate::run(
                Path::new(output_directory),
                str::parse(thread_count).unwrap(),
            )
        }
        _ => Ok(()),
    }
}
