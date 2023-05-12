use better_panic::Settings;
use cli_sandbox::WithStdout;
use std::process::Command;

#[test]
fn fmt() {
    Settings::new()
        .verbosity(better_panic::Verbosity::Minimal)
        .install();
    let cmd = Command::new("cargo")
        .args(["fmt", "--check"])
        .output()
        .expect("Couldn't check formatting (`cargo fmt --check`)");

    if !cmd.empty_stdout() {
        panic!("`cargo fmt --check` stdout wasn't empty, run `cargo fmt` to format the project");
    };
}
