use dors::run;

#[test]
fn test_workspace_only() {
    ["check", "should-be-on-member"]
        .iter()
        .for_each(|task| assert!(run(task, "./tests/workspace_only").unwrap().success()));
}

#[test]
fn test_workspace_failures() {
    ["should-fail"].iter().for_each(|task| {
        assert_eq!(
            run(task, "./tests/workspace_only").unwrap().code().unwrap(),
            55
        )
    });
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

#[test]
fn test_workspace_all() {
    ["should-not-overwrite", "should-overwrite-members", "nested-works-with-run-variants"]
        .iter()
        .for_each(|task| assert!(run(task, "./tests/workspace_all").unwrap().success()));
}

#[test]
fn test_workspace_all_failures() {
    ["should-overwrite"].iter().for_each(|task| {
        assert_eq!(
            run(task, "./tests/workspace_all").unwrap().code().unwrap(),
            55
        )
    });
    ["should-overwrite"].iter().for_each(|task| {
        assert_eq!(
            run(task, "./tests/workspace_all/member2")
                .unwrap()
                .code()
                .unwrap(),
            55
        )
    });
}

#[test]
fn test_workspace_all_member1() {
    ["should-overwrite"].iter().for_each(|task| {
        assert!(run(task, "./tests/workspace_all/member1")
            .unwrap()
            .success())
    });
}
