use dors::run;

#[test]
fn test_workspace_only() {
    assert!(run("check", "./tests/workspace_only").unwrap().success());
}

#[test]
fn test_member_only() {
    assert!(run("check", "./tests/workspace_member_only/member1")
        .unwrap()
        .success());
}

#[test]
fn test_member_only_on_workspace_root() {
    assert!(run(
        "should-be-in-workspace",
        "./tests/workspace_member_only/member1"
    )
    .unwrap()
    .success());
}
