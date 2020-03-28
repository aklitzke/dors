use clap::{crate_version, App, Arg, ArgGroup, SubCommand};
use colored::Colorize;

fn main() {
    std::process::exit(app());
}

fn app() -> i32 {
    let about = "No-fuss workspace-aware task runner for rust";
    let app_matches = App::new("dors -- do things, for rust!")
        .bin_name("cargo")
        .version(crate_version!())
        .author("Andrew Klitzke <nafango2@gmail.com>")
        .about(about)
        .subcommand(
            SubCommand::with_name("dors")
                .about(about)
                .arg(
                    Arg::with_name("list")
                        .short("l")
                        .long("list")
                        .help("list all the available tasks"),
                )
                .arg(Arg::with_name("TASK").help("the name of the task to run"))
                .group(
                    ArgGroup::with_name("run")
                        .args(&["list", "TASK"])
                        .required(true),
                ),
        )
        .get_matches();

    let (subcommand_name, matches_opt) = app_matches.subcommand();
    if subcommand_name != "dors" {
        println!(
            "{}: must invoke dors as `{}`",
            "Error".red(),
            "cargo dors".bold()
        );
        return 1;
    }

    let matches = matches_opt.unwrap();

    if matches.is_present("list") {
        let mut tasks = dors::all_tasks(std::env::current_dir().unwrap()).unwrap();
        tasks.sort();
        tasks.iter().for_each(|task| println!("{}", task));
        return 0;
    }
    if let Some(task) = matches.value_of("TASK") {
        return dors::run(&task, std::env::current_dir().unwrap())
            .unwrap()
            .code()
            .unwrap();
    }
    1
}
