mod dorsfile;
mod util;
use cargo_metadata::MetadataCommand;
use dorsfile::{Dorsfile, Run, Task};
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::ExitStatus;
use util::TakeWhileOkExt;

fn merge_dorsfiles(mut curr: Dorsfile, workspace: &Dorsfile) -> Dorsfile {
    let mut env = workspace.env.clone();
    let mut task = workspace.task.clone();
    env.extend(curr.env.drain());
    task.extend(curr.task.drain());
    curr.env = env;
    curr.task = task;
    curr
}

struct DorsfileGetter {
    workspace_root: PathBuf,
    workspace_dorsfile: Option<Dorsfile>,
}
impl DorsfileGetter {
    pub fn new<P: AsRef<Path>>(workspace_root: P) -> Result<DorsfileGetter, Box<dyn Error>> {
        let workspace_dorsfile_path = workspace_root.as_ref().join("./Dorsfile.toml");
        Ok(DorsfileGetter {
            workspace_root: workspace_root.as_ref().into(),
            workspace_dorsfile: if workspace_dorsfile_path.exists() {
                Some(Dorsfile::load(&workspace_dorsfile_path)?)
            } else {
                None
            },
        })
    }

    pub fn get<P: AsRef<Path>>(&self, crate_path: P) -> Result<Dorsfile, Box<dyn Error>> {
        println!(
            "getting: {:?}\nworkspace_root: {:?}",
            crate_path.as_ref(),
            self.workspace_root
        );
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

pub fn run<P: AsRef<Path>>(task: &str, dir: P) -> Result<ExitStatus, Box<dyn Error>> {
    let dir = dir.as_ref().canonicalize().unwrap();
    println!("running from dir: {:?}", dir);
    let metadata = MetadataCommand::new().current_dir(&dir).exec().unwrap();
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

    let dorsfile = dorsfiles.get(&dir).unwrap();

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
        println!("Running task {:?} in {:?}", task_name, dir);
        let task = dorsfile
            .task
            .get(task_name)
            .ok_or(format!("no task '{}' defined", task_name))?;

        Ok(match task.run {
            Run::Here => run_command(&task.command, dir, &dorsfile.env),
            Run::WorkspaceRoot => {
                // TODO error gracefully when someone messes this up
                // run_command(task, metadata.workspace_root, dorsfile.env)
                run_command(&task.command, &task_runner.workspace_root, &dorsfile.env)
            }
            Run::AllMembers => {
                if dir != task_runner.workspace_root {
                    panic!("cannot run on all-members from outside workspace root");
                }
                task_runner
                    .workspace_paths
                    .iter()
                    .map(|path| {
                        let mut dorsfile = task_runner.dorsfiles.get(&path)?;
                        // avoid infinite loop
                        if let AllMembers = &dorsfile.task[task_name].run {
                            dorsfile.task.get_mut(task_name).unwrap().run = Run::Here
                        }
                        run_task(task_name, &dorsfile, &path, task_runner)
                    })
                    .take_while_ok()
                    .last()
                    .unwrap()?
            }
            Run::Path(ref target_path) => {
                run_command(&task.command, dir.join(target_path), &dorsfile.env)
            }
        })
    }

    run_task(
        task,
        &dorsfile,
        &dir,
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
    let file = Path::new("./")
        .canonicalize()
        .unwrap()
        .join(format!("tmp-{}.sh", chars));
    std::fs::write(&file, command).unwrap();
    println!("{:?}", workdir.as_ref().canonicalize());
    let exit_status = Command::new("bash")
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
    run("check", ".");
}
