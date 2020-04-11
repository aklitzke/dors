use crate::error::{DorsError, Error};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug, Clone)]
pub struct Dorsfile {
    #[serde(default)]
    pub env: Vec<HashMap<String, String>>,
    #[serde(default)]
    pub task: HashMap<String, Task>,
}

#[serde(deny_unknown_fields)]
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Task {
    #[serde(default)]
    pub run_from: Run,
    #[serde(default)]
    pub command: String,
    pub before: Option<Vec<String>>,
    pub after: Option<Vec<String>>,
    #[serde(flatten)]
    pub member_modifiers: Option<MemberModifiers>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum MemberModifiers {
    SkipMembers(HashSet<String>),
    OnlyMembers(HashSet<String>),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum Run {
    Here,
    Path(PathBuf),
    WorkspaceRoot,
    Members,
}
impl Default for Run {
    fn default() -> Run {
        Run::Here
    }
}

impl Dorsfile {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Dorsfile, Box<dyn Error>> {
        let file = match read_to_string(path.as_ref()) {
            Ok(file) => file,
            Err(e) => {
                return Err(match e.kind() {
                    std::io::ErrorKind::NotFound => DorsError::NoDorsfile,
                    _ => DorsError::Unknown(e.into()),
                }
                .into())
            }
        };
        Self::parse(file.as_str())
    }
    pub fn parse(s: &str) -> Result<Dorsfile, Box<dyn Error>> {
        Ok(toml::from_str(s).map_err(DorsError::CouldNotParseDorsfile)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_dorsfile() {
        let sample = r#"
[[env]]
HI = "HOW ARE YOU"

[task.build]
command = "cargo build"
run-from = "here"

[task.check]
command = "cargo check"

[task.long]
command = '''
    hi one
    hi two
'''
run-from = "members"
skip-members = ["member1"]

[task.skip]
command = "echo hi"
only-members = ["member2"]

[task.specific]
command = "echo 'hi'"
run-from = { path = "../whaat" }

[task.empty]
"#;
        let mf = Dorsfile::parse(sample).unwrap();
        assert_eq!(mf.task.len(), 6);
        assert_eq!(mf.env.len(), 1);
    }
}
