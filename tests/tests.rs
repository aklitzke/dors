use dors::run;

#[test]
fn test_workspace() {
    run("check", "./workspace").unwrap();
}
