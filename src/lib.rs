#![deny(clippy::print_stdout)]
mod dorsfile;
mod error;
mod take_while_ext;

pub use crate::error::{DorsError, Error};

use cargo_metadata::MetadataCommand;
use colored::Colorize;
use dorsfile::{Dorsfile, MemberModifiers, Run};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::ExitStatus;
use take_while_ext::TakeWhileLastExt;

#[derive(Debug)]
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
        let crate_path = crate_path.as_ref();
        if crate_path.canonicalize().unwrap() == self.workspace_root.canonicalize().unwrap() {
            return Ok(self
                .workspace_dorsfile
                .as_ref()
                .cloned()
                .ok_or(DorsError::NoDorsfile)?);
        }
        let local = crate_path.join("./Dorsfile.toml");

        let mut dorsfile = match (local.exists(), self.workspace_dorsfile.is_some()) {
            (true, true) => {
                // Extend local dorsfile with workspace dorsfile
                let mut curr = Dorsfile::load(local)?;
                let workspace_dorsfile = self.workspace_dorsfile.as_ref().unwrap();
                let mut env = workspace_dorsfile.env.clone();
                let mut task = workspace_dorsfile.task.clone();

                task.values_mut().for_each(|task| {
                    // Clear all befores and afters from member task
                    // so that they are not ran on both member and workspace root
                    task.before = None;
                    task.after = None;

                    // Clear any 'run-from = "member"' from the workspace, as we ARE running
                    // from the member
                    if let Run::Members = task.run_from {
                        task.run_from = Run::Here;
                    }
                });

                env.extend(curr.env.drain(..));
                task.extend(curr.task.drain());
                curr.env = env;
                curr.task = task;
                curr
            }
            (true, false) => Dorsfile::load(local)?,
            (false, true) => {
                let mut curr = self.workspace_dorsfile.as_ref().cloned().unwrap();

                let mut task = curr.task.clone();
                task.values_mut().for_each(|task| {
                    // Clear all befores and afters from member task
                    // so that they are not ran on both member and workspace root
                    task.before = None;
                    task.after = None;

                    // Clear any 'run-from = "member"' from the workspace, as we ARE running
                    // from the member
                    if let Run::Members = task.run_from {
                        task.run_from = Run::Here;
                    }
                });

                curr.task = task;
                curr
            }
            (false, false) => return Err(DorsError::NoMemberDorsfile.into()),
        };

        // extend environment
        let builtins: HashMap<_, _> = [(
            "CARGO_WORKSPACE_ROOT",
            self.workspace_root.to_str().unwrap(),
        )]
        .iter()
        .cloned()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
        let mut env = vec![builtins];
        env.extend(dorsfile.env.drain(..));
        dorsfile.env = env;
        Ok(dorsfile)
    }
}

struct CargoWorkspaceInfo {
    members: HashMap<String, PathBuf>,
    root: PathBuf,
}

impl CargoWorkspaceInfo {
    fn new(dir: &Path) -> CargoWorkspaceInfo {
        let metadata = MetadataCommand::new().current_dir(&dir).exec().unwrap();
        let root = metadata.workspace_root;
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
                    package.manifest_path.parent().unwrap().into(),
                )
            })
            .collect();
        CargoWorkspaceInfo { members, root }
    }
}

fn run_command(
    command: &str,
    workdir: &Path,
    env: &[HashMap<String, String>],
    args: &[String],
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
    let mut script = env
        .iter()
        .flatten()
        .fold("".to_string(), |mut acc, (k, v)| {
            acc.push_str(&format!("export {}={}\n", k, v));
            acc
        });
    script.push_str(command);
    script.push_str("\n");
    std::fs::write(&file, &script).unwrap();
    let exit_status = Command::new("bash")
        .arg("-e")
        .arg(file.to_str().unwrap())
        .args(args)
        .current_dir(workdir)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    std::fs::remove_file(file).unwrap();
    exit_status
}

pub fn all_tasks<P: AsRef<Path>>(dir: P) -> Result<Vec<String>, Box<dyn Error>> {
    let workspace = CargoWorkspaceInfo::new(dir.as_ref());
    let dorsfiles = DorsfileGetter::new(&workspace.root)?;
    Ok(dorsfiles
        .get(dir.as_ref())?
        .task
        .keys()
        .cloned()
        .collect::<Vec<_>>())
}

fn print_task(task_name: &str, path: &Path) {
    // TODO convert absolute path to relative
    eprintln!(
        "      {} Running {} from `{}`",
        "[Dors]".yellow().bold(),
        task_name.bold(),
        path.to_str().unwrap().bold()
    );
}

pub fn run<P: AsRef<Path>>(task: &str, dir: P) -> Result<ExitStatus, Box<dyn Error>> {
    run_with_args(task, dir, &[])
}

struct TaskRunner {
    workspace: CargoWorkspaceInfo,
    dorsfiles: DorsfileGetter,
}

pub fn run_with_args<P: AsRef<Path>>(
    task: &str,
    dir: P,
    args: &[String],
) -> Result<ExitStatus, Box<dyn Error>> {
    let dir = dir.as_ref();
    let workspace = CargoWorkspaceInfo::new(&dir);
    let dorsfiles = DorsfileGetter::new(&workspace.root)?;
    let dorsfile = dorsfiles.get(&dir)?;

    TaskRunner {
        workspace,
        dorsfiles,
    }
    // seed recursion
    .run_task(
        task,
        &dorsfile,
        &dir,
        args,
        &mut HashSet::new(),
        &mut HashSet::new(),
    )
}

impl TaskRunner {
    fn run_task(
        &self,
        task_name: &str,
        dorsfile: &Dorsfile,
        dir: &Path,
        args: &[String],
        already_ran_befores: &mut HashSet<String>,
        already_ran_afters: &mut HashSet<String>,
    ) -> Result<ExitStatus, Box<dyn Error>> {
        let task = dorsfile
            .task
            .get(task_name)
            .ok_or_else(|| DorsError::NoTask(task_name.to_string()))?;

        // Handle befores
        if let Some(ref befores) = task.before {
            if let Some(befores_result) = befores
                .iter()
                .filter_map(|before_task_name| {
                    if !already_ran_befores.contains(before_task_name) {
                        already_ran_befores.insert(before_task_name.into());
                        Some(self.run_task(
                            before_task_name,
                            dorsfile,
                            dir,
                            &[],
                            already_ran_befores,
                            &mut HashSet::new(),
                        ))
                    } else {
                        None
                    }
                })
                .take_while_last(|result| result.is_ok() && result.as_ref().unwrap().success())
                .last()
            {
                if befores_result.is_err() || !befores_result.as_ref().unwrap().success() {
                    return befores_result;
                }
            }
        }

        // run command
        let result = match task.run_from {
            Run::Here => {
                print_task(task_name, &dir);
                run_command(&task.command, dir, &dorsfile.env, args)
            }
            Run::WorkspaceRoot => {
                // TODO error gracefully when someone messes this up
                let path = &self.workspace.root;
                print_task(task_name, path);
                run_command(&task.command, path, &dorsfile.env, args)
            }
            Run::Members => {
                if dir.canonicalize().unwrap() != self.workspace.root.canonicalize().unwrap() {
                    panic!("cannot run from members from outside workspace root");
                }
                self.workspace
                    .members
                    .iter()
                    .filter_map(|(name, path)| {
                        let short_path = if path.is_relative() {
                            path
                        } else {
                            path.strip_prefix(&self.workspace.root).unwrap()
                        };
                        match task.member_modifiers {
                            Some(ref modifiers) => match modifiers {
                                MemberModifiers::SkipMembers(skips) => {
                                    if skips.contains(name)
                                        || skips.contains(&short_path.to_str().unwrap().to_string())
                                    {
                                        None
                                    } else {
                                        Some(path)
                                    }
                                }
                                MemberModifiers::OnlyMembers(onlys) => {
                                    if onlys.contains(name)
                                        || onlys.contains(&short_path.to_str().unwrap().to_string())
                                    {
                                        Some(path)
                                    } else {
                                        None
                                    }
                                }
                            },
                            None => Some(path),
                        }
                    })
                    .map(|path| {
                        let dorsfile = self.dorsfiles.get(&path)?;
                        self.run_task(
                            task_name,
                            &dorsfile,
                            &path,
                            args,
                            &mut HashSet::new(),
                            &mut HashSet::new(),
                        )
                    })
                    .take_while_last(|result| result.is_ok() && result.as_ref().unwrap().success())
                    .last()
                    .unwrap()?
            }
            Run::Path(ref target_path) => {
                print_task(task_name, &target_path);
                run_command(&task.command, &dir.join(target_path), &dorsfile.env, args)
            }
        };

        if !result.success() {
            return Ok(result);
        }

        // handle afters
        if let Some(ref afters) = task.after {
            if let Some(afters_result) = afters
                .iter()
                .filter_map(|after_task_name| {
                    if !already_ran_afters.contains(after_task_name) {
                        already_ran_afters.insert(after_task_name.into());
                        Some(self.run_task(
                            after_task_name,
                            dorsfile,
                            dir,
                            &[],
                            &mut HashSet::new(),
                            already_ran_afters,
                        ))
                    } else {
                        None
                    }
                })
                .take_while_last(|result| result.is_ok() && result.as_ref().unwrap().success())
                .last()
            {
                if afters_result.is_err() || !afters_result.as_ref().unwrap().success() {
                    return afters_result;
                }
            }
        }

        Ok(result)
    }
}
