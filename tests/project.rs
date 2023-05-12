use better_panic;
use cli_sandbox::project;
use pretty_assertions::assert_ne;

#[test]
fn new_project() {
    better_panic::install();
    let _ = project().expect("Couldn't create a new project");
}

#[test]
fn multiple_projects_different_paths() {
    better_panic::install();
    let proj1 = project().expect("Couldn't create a new project");
    let proj2 = project().expect("Couldn't create a new project");

    assert_ne!(proj1.path(), proj2.path());
}
