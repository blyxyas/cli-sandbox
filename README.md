# `cli-sandbox`

`cli-sandbox` is a sandboxing environment and testing utility to help you test and debug your CLI applications, inspired by [Cargo's `cargo-test-support`](https://github.com/rust-lang/cargo/tree/master/crates/cargo-test-support).

## Features

* Pretty assertions, powered by [`pretty_assertions`](https://docs.rs/pretty_assertions/latest/pretty_assertions/). (*optional*)
* Testing for either channel, `dev` or `release`.
* 

## Why?

The best way to test an application is by simulating a user's environment. `cli-sandbox` creates a temporary directory for each test, and redirects and IO into that designated directory.

Checking a tool `init` command is as simple as this:

```rs
#[test]
fn init_doc() -> Result<()> {
    let proj = Project::new()?;
	let cmd = proj.command(["init"])?;
	cmd.with_stderr(""); // There shouldn't be any errors
	cmd.with_stdout("The project was initialized! :D");
proj.check_file(Path::new("<The path where a file was generated>"), r#"WHAT SHOULD APPEAR IN THE FILE"#)?;
	Ok(())
}
```