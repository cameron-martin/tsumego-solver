mod explore;
mod generate;

use clap::{App, AppSettings, Arg, SubCommand};

fn main() {
    let matches = App::new("Tsumego Solver")
        .subcommand(
            SubCommand::with_name("explore").arg(
                Arg::with_name("file")
                    .short("f")
                    .long("file")
                    .required(true)
                    .takes_value(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("generate"), /*.arg(
                                                   Arg::with_name("out")
                                                       .short("o")
                                                       .long("out")
                                                       .required(true)
                                                       .takes_value(true),
                                               )*/
        )
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    match matches.subcommand() {
        ("explore", Some(matches)) => {
            let filename = matches.value_of("file").unwrap();

            explore::run(filename);
        }
        ("generate", _) => {
            generate::run();
        }
        _ => {}
    }
}
