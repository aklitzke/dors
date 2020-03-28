mod dorsfile;

use cargo_metadata::MetadataCommand;
use dorsfile::{Dorsfile, Run, Task};
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::ExitStatus;

fn merge_dorsfiles(mut curr: Dorsfile, workspace: &Dorsfile) -> Dorsfile {
    let mut env = workspace.env.clone();
    let mut task = workspace.task.clone();
    env.extend(curr.env.drain());
    task.extend(curr.task.drain());
    curr.env = env;
    curr.task = task;
    curr
}

pub struct DorsfileGetter {
    workspace_root: PathBuf,
    workspace_dorsfile: Option<Dorsfile>,
}
impl DorsfileGetter {
    pub fn new<P: AsRef<Path>>(workspace_root: P) -> Result<DorsfileGetter, Box<dyn Error>> {
        let workspace_dorsfile_path = workspace_root.as_ref().join("./Dorsfile.toml");
        Ok(DorsfileGetter {
            workspace_root: workspace_root.as_ref().into(),
            workspace_dorsfile: match workspace_dorsfile_path.exists() {
                true => Some(Dorsfile::load(&workspace_dorsfile_path)?),
                false => None,
            },
        })
    }

    pub fn get<P: AsRef<Path>>(&self, crate_path: P) -> Result<Dorsfile, Box<dyn Error>> {
        if crate_path.as_ref() == self.workspace_root {
            return Ok(self
                .workspace_dorsfile
                .as_ref()
                .cloned()
                .ok_or("no workspace dorsfile")?);
        }
        let local = crate_path.as_ref().join("./Dorsfile.toml");

        Ok(match (local.exists(), self.workspace_dorsfile.is_some()) {
            (true, true) => merge_dorsfiles(
                Dorsfile::load(local)?,
                self.workspace_dorsfile.as_ref().unwrap(),
            ),
            (true, false) => Dorsfile::load(local)?,
            (false, true) => self.workspace_dorsfile.as_ref().cloned().unwrap(),
            (false, false) => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find Dorsfile.toml",
                )))
            }
        })
    }
}

fn run(task: &str) -> Result<ExitStatus, Box<dyn Error>> {
    let metadata = MetadataCommand::new().exec().unwrap();
    let workspace_root = metadata.workspace_root.canonicalize().unwrap();
    // allow O(1) referencing of package information
    let packages: HashMap<_, _> = metadata
        .packages
        .iter()
        .map(|package| (package.id.clone(), package))
        .collect();
    let workspace_paths: Vec<PathBuf> = metadata
        .workspace_members
        .into_iter()
        .map(|member| {
            packages[&member]
                .manifest_path
                .canonicalize()
                .unwrap()
                .parent()
                .unwrap()
                .into()
        })
        .collect();
    println!("workspace_paths: {:#?}", workspace_paths);
    let dorsfiles = DorsfileGetter::new(&workspace_root).unwrap();

    let current_dir = std::env::current_dir().unwrap();

    let dorsfile = dorsfiles.get(&current_dir).unwrap();

    struct TaskRunner {
        workspace_paths: Vec<PathBuf>,
        workspace_root: PathBuf,
        dorsfiles: DorsfileGetter,
    }

    fn run_task(
        task_name: &str,
        dorsfile: &Dorsfile,
        dir: &Path,
        task_runner: &TaskRunner,
    ) -> Result<ExitStatus, Box<dyn Error>> {
        let task = dorsfile.task.get(task_name).unwrap();

        Ok(match task.run {
            Run::Here => run_command(&task.command, dir, &dorsfile.env),
            Run::WorkspaceRoot => {
                // TODO error gracefully when someone messes this up
                // run_command(task, metadata.workspace_root, dorsfile.env)
                run_command(&task.command, &PathBuf::new(), &dorsfile.env)
            }
            Run::AllMembers => {
                // workspace_members
                // check if I'm at workspace root -- error if not
                // generate list of members to run against
                // Load dorsfile for each of these members
                // call run task against each member,
                // setting task = the name of this task,
                // dorsfile = the thing I loaded for that dir
                // dir equal to the dir of that workspace
                if dir != task_runner.workspace_root {
                    panic!("cannot run on all-members from outside workspace root");
                }
                task_runner
                    .workspace_paths
                    .iter()
                    // load dorsfiles
                    .map(|path| {
                        let dorsfile = task_runner.dorsfiles.get(&path)?;
                        run_task(task_name, &dorsfile, &path, task_runner)
                    })
                    .take_while(|result| result.is_ok() && result.as_ref().unwrap().success())
                    .last()
                    .unwrap()?
            }
            _ => panic!("unhandled run method"),
        })
    }

    run_task(
        task,
        &dorsfile,
        &current_dir,
        &TaskRunner {
            workspace_root,
            workspace_paths,
            dorsfiles,
        },
    )
}

fn run_command<P: AsRef<Path>>(
    command: &str,
    workdir: P,
    env: &HashMap<String, String>,
) -> ExitStatus {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::iter;
    let mut rng = thread_rng();
    let chars: String = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(10)
        .collect();
    let file = PathBuf::from(format!("tmp-{}.sh", chars));
    std::fs::write(&file, command).unwrap();
    let exit_status = Command::new("sh")
        .arg(file.to_str().unwrap())
        .envs(env)
        .current_dir(workdir)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    std::fs::remove_file(file).unwrap();
    exit_status
}

#[test]
fn test() {
    run("check");
}
