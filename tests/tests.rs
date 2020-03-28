use dors::run;

#[test]
fn test_workspace_only() {
    assert!(run("check", "./tests/workspace_only").unwrap().success());
}

#[test]
fn test_member_only() {
    [
        "should-be-here",
        "should-be-here-explicit",
        "should-be-in-workspace",
        "should-be-in-tests",
    ]
    .iter()
    .for_each(|task| {
        assert!(run(task, "./tests/workspace_member_only/member1")
            .unwrap()
            .success())
    });
}
