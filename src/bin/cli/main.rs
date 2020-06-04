mod generate;

use clap::{App, AppSettings, Arg, SubCommand};
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    let matches = App::new("Tsumego Solver")
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
