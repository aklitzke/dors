use clap::{App, Arg, ArgGroup, SubCommand};
use colored::Colorize;

fn main() {
    std::process::exit(app());
}

fn app() -> i32 {
    let about = "No-fuss workspace-aware task runner for rust";
    let app_matches = App::new("dors -- do things, for rust!")
        .bin_name("cargo")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Andrew Klitzke <andrewknpe@gmail.com>")
        .about(about)
        .subcommand(
            SubCommand::with_name("dors")
                .version(env!("CARGO_PKG_VERSION"))
                .about(about)
                .arg(
                    Arg::with_name("list")
                        .short("l")
                        .long("list")
                        .help("list all the available tasks"),
                )
                .arg(Arg::with_name("TASK").help("the name of the task to run"))
                .group(ArgGroup::with_name("run").args(&["list", "TASK"])),
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
        let mut tasks = match dors::all_tasks(std::env::current_dir().unwrap()) {
            Ok(tasks) => tasks,
            Err(e) => {
                println!("{}", e);
                return 1;
            }
        };
        tasks.sort();
        tasks.iter().for_each(|task| println!("{}", task));
        return 0;
    }
    if let Some(task) = matches.value_of("TASK") {
        match dors::run(&task, std::env::current_dir().unwrap()) {
            Ok(resp) => return resp.code().unwrap(),
            Err(e) => {
                println!("{}", e);
                return 1;
            }
        }
    }

    let mut tasks = match dors::all_tasks(std::env::current_dir().unwrap()) {
        Ok(tasks) => tasks,
        Err(e) => {
            println!("{}", e);
            return 1;
        }
    };
    tasks.sort();

    if matches.is_present("list") {
        tasks.iter().for_each(|task| println!("{}", task));
        return 0;
    }

    println!("{}: Please select a task to run:", "Error".red());
    tasks.iter().for_each(|task| println!("{}", task.bold()));
    1
}
