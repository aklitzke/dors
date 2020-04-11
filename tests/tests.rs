use dors::DorsError;
use dors::{all_tasks, run, run_with_args};

#[test]
fn test_workspace_only() {
    [
        "check",
        "should-be-on-member",
        "should-run-before-only-once",
        "should-run-after-only-once",
        "should-not-run-befores-on-members",
    ]
    .iter()
    .for_each(|task| assert!(run(task, "./tests/workspace_only").unwrap().success()));
}

#[test]
fn test_workspace_failures() {
    ["should-fail", "should-fail-in-multiline"]
        .iter()
        .for_each(|task| {
            assert_eq!(
                run(task, "./tests/workspace_only").unwrap().code().unwrap(),
                55
            )
        });
}

#[test]
fn test_workspace_failures_from_member() {
    [
        "should-fail",
        "should-fail-in-multiline",
        "fail-if-not-on-root",
    ]
    .iter()
    .for_each(|task| {
        assert_eq!(
            run(task, "./tests/workspace_only/member1")
                .unwrap()
                .code()
                .unwrap(),
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
        "should-be-one",
        "should-be-one-at-root",
        "should-have-default-env",
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
    [
        "should-not-overwrite",
        "should-overwrite-members",
        "nested-works-with-run-variants",
        "should-not-run-before-or-after-on-member",
        "only-member1",
        "only-member2",
        "should-inherit-envs",
        "should-have-no-args",
    ]
    .iter()
    .for_each(|task| {
        let result = run(task, "./tests/workspace_all").unwrap().success();
        assert!(result);
    });
}

#[test]
fn test_workspace_all_args() {
    assert!(run_with_args(
        "should-pass-args",
        "tests/workspace_all",
        &["".to_string(), "2".to_string()]
    )
    .unwrap()
    .success());
}

#[test]
fn test_workspace_all_failures() {
    ["should-overwrite", "should-fail", "should-pass-args"]
        .iter()
        .for_each(|task| {
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

#[test]
fn test_list_workspace_all() {
    let mut all_tasks = all_tasks("./tests/workspace_all").unwrap();
    all_tasks.sort();
    assert_eq!(
        all_tasks,
        [
            "check",
            "nested-works-with-run-variants",
            "only-member1",
            "only-member2",
            "should-fail",
            "should-have-no-args",
            "should-inherit-envs",
            "should-not-overwrite",
            "should-not-run-before-or-after-on-member",
            "should-overwrite",
            "should-overwrite-members",
            "should-pass-args",
        ]
    );
}

#[test]
fn test_list_member_only() {
    let all_tasks = all_tasks("./tests/workspace_member_only/member1").unwrap();
    assert_eq!(all_tasks.len(), 7);
}

#[test]
fn test_no_task() {
    let err = run("fake-task", "tests/workspace_all").unwrap_err();
    assert!(matches!(
        err.kind(),
        DorsError::NoTask(task_name) if task_name == "fake-task"
    ));
}

#[test]
fn test_workspace_only_from_member() {
    ["should-be-on-member", "should-run-before-only-once"]
        .iter()
        .for_each(|task| {
            assert!(run(task, "./tests/workspace_only/member1")
                .unwrap()
                .success())
        });
}

#[test]
fn test_no_dorsfiles() {
    // since writing is occurring, careful not to use ths dir outside this test!
    let tmp_file = "tests/no_dorsfiles/Dorsfile.toml";

    assert!(matches!(
        all_tasks("./tests/no_dorsfiles").unwrap_err().kind(),
        DorsError::NoDorsfile
    ));

    assert!(matches!(
        run("", "tests/no_dorsfiles/member1").unwrap_err().kind(),
        DorsError::NoMemberDorsfile
    ));

    std::fs::write(tmp_file, b"invalid-syntax").unwrap();
    assert!(matches!(
        all_tasks("tests/no_dorsfiles").unwrap_err().kind(),
        DorsError::CouldNotParseDorsfile(_)
    ));

    std::fs::write(tmp_file, b"[task.a]\nunexpected-field=1").unwrap();
    assert!(matches!(
        all_tasks("tests/no_dorsfiles").unwrap_err().kind(),
        DorsError::CouldNotParseDorsfile(_)
    ));

    std::fs::remove_file(tmp_file).unwrap();
}
