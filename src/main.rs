use clap::{crate_version, App, Arg, ArgGroup};

fn main() {
    let matches = App::new("Dors -- do things, for rust!")
        .version(crate_version!())
        .author("Andrew Klitzke <nafango2@gmail.com>")
        .about("Workspace-friendly task runner for cargo")
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
        )
        .get_matches();

    if matches.is_present("list") {
        let mut tasks = dors::all_tasks(std::env::current_dir().unwrap()).unwrap();
        tasks.sort();
        tasks.iter().for_each(|task|
            println!("{}", task)
        );
        return;
    }
    if let Some(task) = matches.value_of("TASK") {
        dors::run(&task, std::env::current_dir().unwrap()).unwrap();
    }
}
