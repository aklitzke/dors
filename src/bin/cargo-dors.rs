use clap::{App, SubCommand};
use colored::Colorize;

fn main() {
    std::process::exit((|| {
        let app_matches = App::new("dors -- do things, for rust!")
            .bin_name("cargo")
            .version(env!("CARGO_PKG_VERSION"))
            .author("Andrew Klitzke <andrewknpe@gmail.com>")
            .about(dors::get_about())
            .max_term_width(80)
            .subcommand(dors::set_app_options(SubCommand::with_name("dors")))
            .get_matches();

        let (subcommand_name, matches_opt) = app_matches.subcommand();
        if subcommand_name != "dors" {
            println!(
                "{}: must invoke dors as `{}` or `{}`",
                "Error".red(),
                "cargo dors".bold(),
                "dors".bold(),
            );
            return 1;
        }

        let matches = matches_opt.unwrap();

        dors::process_cmd(matches)
    })());
}
