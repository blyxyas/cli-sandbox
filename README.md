<span align="center">

<h1><pre><code>cli-sandbox</code></pre></h1>

<a href="https://crates.io/crates/cli-sandbox"><img src="https://img.shields.io/crates/d/cli-sandbox?style=for-the-badge&logo=rust"></img></a>
<a href="https://docs.rs/cli-sandbox"><img src="https://img.shields.io/docsrs/cli-sandbox?style=for-the-badge&logo=docsdotrs"></img></a>

</span>

`cli-sandbox` is a sandboxing environment and testing utility to help you test and debug your CLI applications, inspired by [Cargo's `cargo-test-support`](https://github.com/rust-lang/cargo/tree/master/crates/cargo-test-support).

All tests get their own temporary directories, where you can create files, check files, test your program against those files and check the output of your program in various ways.

For example, if you want to check that your Python to Rust transpiler works correctly:

```rust
use cli_sandbox::{project, WithStdout};
use std::error::Error;

#[test]
fn compiling() -> Result<(), Box<dyn Error>> {
	let proj = project()?;                      // Create a project

	// Let's create a file, and put in there some Python.
	proj.new_file("my-program.py",
r#"def main():
	print("Hi! this is a test")

main()"#)?;

	let cmd = proj.command(["build"])?;         // Execute the command "<YOUR COMMAND> build". Cli-sandbox will automatically get pickup your command.

	// Now, let's check that the transpiler created the file correctly.
	proj.check_file("my-program.rs", 
r#"fn main() {
	println!("Hi! this is a test");
}

main()"#)?;

	// And that the command stdout and stderr are correct.

	cmd.with_stdout("File transpiled correctly! (`my-program.py` -> `my-program.rs`)");

	// If the stderr isn't empty, we'll panic.
	if !cmd.empty_stderr() {
		panic!("Something went wrong! stderr isn't empty");
	};
}
```

You can also get the path of a project (it changes each time the tests are executed, they're temporary).

## Installation

```sh
cargo add cli-sandbox
```

## Usage

The first step is to create a `Project`. You can use either `Project::new()` or `project()`. This will create a temporary directory for you to put all your testing files in there.

From a project, you can execute commands, do I/O operations or even operate over it manually by getting the project's path (`Project::path()`).

Check the [project's documentation](https://docs.rs/cli-sandbox) for more info.

## Features

* Regex support for checking `stdout` and `stderr`. (feature: `regex`)
* All output is beautiful thanks to [`pretty-assertions`](https://docs.rs/pretty_assertions/latest/pretty_assertions/) and [`better_panic`](https://docs.rs/better_panic). (feature: `pretty`, also can be enabled individually)
* Little fuzzing functionality (feature: `fuzz`)
* Testing either the `debug` or `release` profile (features: `dev` or `release`)

## Contributing

Check the [`CONTRIBUTING.md`] file for some guidelines on how to contribute. **All contributions are welcomed, any size from contributors with all levels of experience!**

## LICENSE

we use the **MIT License**, check the [LICENSE] file for more information