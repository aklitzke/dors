# 🌈 _**Dors more, with Dorsfiles**_ 🌂
[![crates.io](https://img.shields.io/crates/v/dors.svg)](https://crates.io/crates/dors)
[![Build Status](https://travis-ci.org/aklitzke/dors.svg?branch=master)](https://travis-ci.org/aklitzke/dors)

**What is this?**

A [task runner](https://en.wikipedia.org/wiki/Build_automation) for
the [rust](https://www.rust-lang.org/) and [cargo](https://github.com/rust-lang/cargo)
ecosystem.

Especially targeted toward environments with a large cargo workspace,
like embedded or cloud-based, that often contain multiple targets, tools
outside of the rust ecosystem, complex deploy scripts, and CI pipelines.

Designed with the hope that easy things will be easy, and hard things will be possible.

## Example

```toml
# ./Dorsfile.toml
[task.do-tests]
command = "cargo test --target x86_86-unknown-linux-gnu -- --nocapture"
```
```bash
$ cargo dors do-tests
```

## Installation

```bash
$ cargo install dors
```

## Features

#### Run tasks on all members of a workspace:
```toml
# ./Dorsfile.toml
[task.test]
command = "echo Hello, World! from $PWD"
run-from = "members"
```

#### Set crate-specific environment variables:
```toml
# ./member-1/Dorsfile
[[env]]
CARGO_TARGET_DIR = "../target-member-1"
```
Dors will automatically assign `CARGO_WORKSPACE_ROOT` for you. $PWD, $HOME, and other
environment variables work as expected.

#### Assign environment variables with bash:
```toml
[[env]]
MY_SPECIAL_ENV_VAR = "$(ls)"
ANOTHER_ENV_VAR = "$HOME/.cargo/bin"
```

#### View all available tasks:
```bash
$ cargo dors -l
my-special-task
load
deploy
```
Also supports tab autocompletion of tasks!

#### Pass arguments:
```toml
[task.say-hi]
command = 'echo Hello, "$@"!'
```
```bash
$ cargo dors say-hi -- Fellow Human
Hello, Fellow Human!
```

#### Run multi-line bash scripts:
```toml
# ./Dorsfile.toml
[task.play-go]
command = '''
url="igs.joyjoy.net 6969"
telnet $url
'''
```

#### Reduce duplication by inheriting tasks:
```toml
# ./Dorsfile.toml
[task.check]
command = "cargo check --all-targets"
```
```bash
$ cd shared_code && cargo dors check
```

#### Invoke tasks before or after others:
```toml
#./Dorsfile.toml
[task.play-go]
before = ["install-telnet"]
command = "telnet igs.joyjoy.net 6969"
after = ["congratulate"]

[task.install-telnet]
command = "sudo apt-get install -y --no-install-recommends telnet"

[task.congratulate]
command = "echo 'I hope you played well!'"
```
Befores/afters are ran from left to right. If a task is repeated in a tree of befores,
it will only be ran once.

#### Override workspace tasks for a single workspace member:
```toml
#./Dorsfile.toml
[task.build]
command = "cargo build"
run-from = "members"
```
```toml
#./embedded_device/Dorsfile.toml
[task.build]
command = "cargo build --features debug-logs"
```

#### Skip particular members:
```toml
#./Dorsfile.toml
[task.test]
command = "cargo test"
run-from = "members"
skip-members = ["embedded_device"]

[task.clippy]
command = "cargo clippy"
run-from = "members"
only-members = ["shared_code"]
```

#### Run commands from member crate on workspace root:
```toml
# ./embedded_device/Dorsfile.toml
[task.pre-build]
run-from = "workspace-root"
command = "echo interestingstuff > target/special-file"
```

#### Run commands from arbitrary paths:
```toml
# ./Dorsfile.toml
[task.run-other-project]
run-from = { path = "../random_node_project" }
command = "npm run"
```

...And more! 🎩

## FAQ:

**Q:** Will this automatically provide me with standard cargo commands?  
**A:** Nope. A blank Dorsfile is a blank Dorsfile. It's up to you to build it out.

**Q:** Is this ready for production?  
**A:** This is currently in alpha. Use at your own risk.

**Q:** This looks pretty similar to [cargo-make](https://github.com/sagiegurari/cargo-make). What are the differences?  
**A:** The syntax was inspired by cargo-make, and this project was originally
started due to limitations in cargo-make's workspace support. This project has different goals, a different task
execution model, and a different syntax.

**Q:** How do you pronounce this?  
**A:** Like 'horse'

**Q:** Shouldn't you just add an 'e' at the end? Like 'dorse'?  
**A:** Never. Nope. Go to your room. Don't come out till you've learned your lesson 🙃

**Q:** Do I have to use this with a workspace?  
**A:** You can use this crate with or without your crate being a workspace.

**Q:** What are the next steps for improvement?  
**A:** Likely error messaging. Want something? Open an issue!

**Q:** Would you accept a PR?  
**A:** Absolutely!

**Q:** My question wasn't answered here  
**A:** Feel free to reach out!

![An open book](https://emojipedia-us.s3.dualstack.us-west-1.amazonaws.com/thumbs/240/apple/237/open-book_1f4d6.png)
