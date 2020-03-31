use clap::{App, AppSettings, Arg, SubCommand};
use colored::Colorize;

fn main() {
    std::process::exit((|| {
        let about = "No-fuss workspace-aware task runner for rust";
        let app_matches = App::new("dors -- do things, for rust!")
            .bin_name("cargo")
            .version(env!("CARGO_PKG_VERSION"))
            .author("Andrew Klitzke <andrewknpe@gmail.com>")
            .about(about)
            .subcommand(
                SubCommand::with_name("dors")
                    .version(env!("CARGO_PKG_VERSION"))
                    .setting(AppSettings::TrailingVarArg)
                    .setting(AppSettings::ColoredHelp)
                    .setting(AppSettings::DontCollapseArgsInUsage)
                    .about(about)
                    .arg(
                        Arg::with_name("list")
                            .short("l")
                            .long("list")
                            .conflicts_with_all(&["TASK", "TASK_ARGS"])
                            .display_order(0)
                            .help("list all the available tasks"),
                    )
                    .arg(Arg::with_name("TASK").help("the name of the task to run"))
                    .arg(
                        Arg::with_name("TASK_ARGS")
                            .help("arguments to pass to the task")
                            .requires("TASK")
                            .multiple(true),
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
            let args = match matches.values_of("TASK_ARGS") {
                Some(values) => values.map(|s| s.to_string()).collect(),
                None => vec![],
            };
            match dors::run_with_args(&task, std::env::current_dir().unwrap(), &args) {
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
    })());
}
