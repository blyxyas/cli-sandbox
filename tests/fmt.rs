use cli_sandbox::WithStdout;
use std::process::Command;

#[test]
fn fmt() {
    let cmd = Command::new("cargo")
        .args(["fmt", "--check"])
        .output()
        .expect("Couldn't check formatting (`cargo fmt --check`)");

    cmd.with_stdout("");
}
