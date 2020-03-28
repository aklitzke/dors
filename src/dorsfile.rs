use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug, Clone)]
pub struct Dorsfile {
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub task: HashMap<String, Task>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Task {
    #[serde(default)]
    pub run: Run,
    pub command: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum Run {
    Here,
    Path(PathBuf),
    WorkspaceRoot,
    AllMembers,
}
impl Default for Run {
    fn default() -> Run {
        Run::Here
    }
}

impl Dorsfile {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Dorsfile, Box<dyn Error>> {
        Self::parse(read_to_string(path.as_ref())?.as_str())
    }
    pub fn parse(s: &str) -> Result<Dorsfile, Box<dyn Error>> {
        Ok(toml::from_str(s)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_dorsfile() {
        let sample = r#"
[env]
HI = "HOW ARE YOU"

[task.build]
command = "cargo build"
run = "here"

[task.check]
command = "cargo check"

[task.long]
command = '''
    hi one
    hi two
'''
run = "all-members"

[task.specific]
command = "echo 'hi'"
run = { path = "../whaat" }
"#;
        let mf = Dorsfile::parse(sample).unwrap();
        println!("{:?}", mf);
        assert_eq!(mf.task.len(), 4);
        assert_eq!(mf.env.len(), 1);
    }
}
