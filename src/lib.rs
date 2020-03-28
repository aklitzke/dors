mod dorsfile;
mod util;
use cargo_metadata::MetadataCommand;
use dorsfile::{Dorsfile, MemberModifiers, Run};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::ExitStatus;
use util::TakeWhileOkExt;

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
            (true, true) => {
                // Extend local dorsfile with workspace dorsfile
                let mut curr = Dorsfile::load(local)?;
                let workspace_dorsfile = self.workspace_dorsfile.as_ref().unwrap();
                let mut env = workspace_dorsfile.env.clone();
                let mut task = workspace_dorsfile.task.clone();
                env.extend(curr.env.drain());
                task.extend(curr.task.drain());
                curr.env = env;
                curr.task = task;
                curr
            }
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

struct CargoWorkspaceInfo {
    members: HashMap<String, PathBuf>,
    root: PathBuf,
}
impl CargoWorkspaceInfo {
    fn new(dir: &Path) -> CargoWorkspaceInfo {
        println!("running from dir: {:?}", dir);
        let metadata = MetadataCommand::new().current_dir(&dir).exec().unwrap();
        let root = metadata.workspace_root.canonicalize().unwrap();
        // allow O(1) referencing of package information
        let packages: HashMap<_, _> = metadata
            .packages
            .iter()
            .map(|package| (package.id.clone(), package))
            .collect();
        let members = metadata
            .workspace_members
            .into_iter()
            .map(|member| {
                let package = packages[&member];
                (
                    package.name.clone(),
                    package
                        .manifest_path
                        .canonicalize()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .into(),
                )
            })
            .collect();
        CargoWorkspaceInfo { members, root }
    }
}

pub fn all_tasks<P: AsRef<Path>>(dir: P) -> Result<Vec<String>, Box<dyn Error>> {
    let workspace = CargoWorkspaceInfo::new(dir.as_ref());
    let dorsfiles = DorsfileGetter::new(&workspace.root).unwrap();
    Ok(dorsfiles
        .get(dir.as_ref())?
        .task
        .keys()
        .cloned()
        .collect::<Vec<_>>())
}

pub fn run<P: AsRef<Path>>(task: &str, dir: P) -> Result<ExitStatus, Box<dyn Error>> {
    let dir = dir.as_ref().canonicalize().unwrap();
    let workspace = CargoWorkspaceInfo::new(&dir);
    let dorsfiles = DorsfileGetter::new(&workspace.root).unwrap();
    let dorsfile = dorsfiles.get(&dir).unwrap();

    struct TaskRunner {
        workspace: CargoWorkspaceInfo,
        dorsfiles: DorsfileGetter,
    }

    fn run_task(
        task_name: &str,
        dorsfile: &Dorsfile,
        dir: &Path,
        already_ran_befores: &mut HashSet<String>,
        already_ran_afters: &mut HashSet<String>,
        task_runner: &TaskRunner,
    ) -> Result<ExitStatus, Box<dyn Error>> {
        println!("Running task {:?} in {:?}", task_name, dir);
        let task = dorsfile
            .task
            .get(task_name)
            .ok_or(format!("no task '{}' defined", task_name))?;

        let mut result: Option<ExitStatus> = None;
        if let Some(ref befores) = task.before {
            // TODO gracefully handle an unknown task name in a before
            if let Some(task_result) = befores
                .iter()
                .filter_map(|before_task_name| {
                    if !already_ran_befores.contains(before_task_name) {
                        already_ran_befores.insert(before_task_name.into());
                        Some(run_task(
                            before_task_name,
                            dorsfile,
                            dir,
                            already_ran_befores,
                            already_ran_afters,
                            task_runner,
                        ))
                    } else {
                        None
                    }
                })
                .take_while_ok()
                .last()
            {
                result.replace(task_result?);
            }
        }

        result.replace(match task.run_from {
            Run::Here => run_command(&task.command, dir, &dorsfile.env),
            Run::WorkspaceRoot => {
                // TODO error gracefully when someone messes this up
                run_command(&task.command, &task_runner.workspace.root, &dorsfile.env)
            }
            Run::Members => {
                if dir != task_runner.workspace.root {
                    panic!("cannot run from members from outside workspace root");
                }
                task_runner
                    .workspace
                    .members
                    .iter()
                    .filter_map(|(name, path)| match task.member_modifiers {
                        Some(ref modifiers) => match modifiers {
                            MemberModifiers::SkipMembers(skips) => {
                                if skips.contains(name) {
                                    None
                                } else {
                                    Some(path)
                                }
                            }
                            MemberModifiers::OnlyMembers(onlys) => {
                                if onlys.contains(name) {
                                    Some(path)
                                } else {
                                    None
                                }
                            }
                        },
                        None => Some(path),
                    })
                    .map(|path| {
                        let mut dorsfile = task_runner.dorsfiles.get(&path)?;
                        // avoid infinite loop
                        if let Run::Members = &dorsfile.task[task_name].run_from {
                            dorsfile.task.get_mut(task_name).unwrap().run_from = Run::Here
                        }
                        run_task(
                            task_name,
                            &dorsfile,
                            &path,
                            &mut HashSet::new(),
                            &mut HashSet::new(),
                            task_runner,
                        )
                    })
                    .take_while_ok()
                    .last()
                    .unwrap()?
            }
            Run::Path(ref target_path) => {
                run_command(&task.command, dir.join(target_path), &dorsfile.env)
            }
        });

        Ok(result.unwrap())
    }

    run_task(
        task,
        &dorsfile,
        &dir,
        &mut HashSet::new(),
        &mut HashSet::new(),
        &TaskRunner {
            workspace,
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
