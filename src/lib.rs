//! <span align="center">
//!
//! <h1><pre><code>cli-sandbox</code></pre></h1>
//!
//! <a href="https://crates.io/crates/cli-sandbox"><img src="https://img.shields.io/crates/d/cli-sandbox?style=for-the-badge&logo=rust"></img></a>
//! <a href="https://docs.rs/cli-sandbox"><img src="https://img.shields.io/docsrs/cli-sandbox?style=for-the-badge&logo=docsdotrs"></img></a>
//!
//! </span>
//!
//! `cli-sandbox` is a sandboxing environment and testing utility to help you test and debug your CLI applications, inspired by [Cargo's `cargo-test-support`](https://github.com/rust-lang/cargo/tree/master/crates/cargo-test-support).
//!
//! All tests get their own temporary directories, where you can create files, check files, test your program against those files and check the output of your program in various ways.
//!
//! For example, if you want to check that your Python to Rust transpiler works correctly:
//!
//! ```rust
//! use cli_sandbox::{project, WithStdout};
//! use std::error::Error;
//!
//! #[test]
//! fn compiling() -> Result<(), Box<dyn Error>> {
//!     cli_sandbox::init(); // Initialize the sandbox
//!     let proj = project()?;                      // Create a project
//!
//!     // Let's create a file, and put in there some Python.
//!     proj.new_file("my-program.py",
//! r#"def main():
//!     print("Hi! this is a test")
//!
//! main()"#)?;
//!
//!     let cmd = proj.command(["build"])?;         // Execute the command "<YOUR COMMAND> build". Cli-sandbox will automatically get pickup your command.
//!
//!     // Now, let's check that the transpiler created the file correctly.
//!     proj.check_file("my-program.rs",
//! r#"fn main() {
//!     println!("Hi! this is a test");
//! }
//!
//! main()"#)?;
//!
//!     // And that the command stdout and stderr are correct.
//!
//!     cmd.with_stdout("File transpiled correctly! (`my-program.py` -> `my-program.rs`)");
//!
//!     // If the stderr isn't empty, we'll panic.
//!     if !cmd.empty_stderr() {
//!         panic!("Something went wrong! stderr isn't empty");
//!     };
//! }
//! ```
//!
//! You can also get the path of a project (it changes each time the tests are executed, they're temporary).
//!
//! ## Installation
//!
//! ```sh
//! cargo add cli-sandbox --dev
//! ```
//!
//! ## Usage
//!
//! The first step is to create a `Project`. You can use either `Project::new()` or `project()`. This will create a temporary directory for you to put all your testing files in there.
//!
//! From a project, you can execute commands, do I/O operations or even operate over it manually by getting the project's path (`Project::path()`).
//!
//! Check the [project's documentation](https://docs.rs/cli-sandbox) for more info.
//!
//! ## Features
//!
//! * Regex support for checking `stdout` and `stderr`. (feature: `regex`)
//! * All output is beautiful thanks to [`pretty-assertions`](https://docs.rs/pretty_assertions/latest/pretty_assertions/) and [`better_panic`](https://docs.rs/better_panic). (feature: `pretty`, also can be enabled individually)
//! * Little fuzzing functionality (feature: `fuzz`)
//! * Testing either the `debug` or `release` profile (features: `dev` or `release`)
//!

// All code blocks in fragments must be ignored because rustdoc hates environment variables, it seems.

#![cfg_attr(feature = "deny-warnings", deny(warnings))] // Use for tests
#![warn(
    unused,
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::undocumented_unsafe_blocks,
    clippy::empty_structs_with_brackets,
    clippy::format_push_string,
    clippy::get_unwrap,
    clippy::if_then_some_else_none,
    clippy::impl_trait_in_params,
    clippy::integer_division,
    clippy::large_include_file,
    clippy::let_underscore_must_use,
    clippy::semicolon_outside_block,
    clippy::str_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::unneeded_field_pattern,
    clippy::use_debug,
    clippy::branches_sharing_code,
    clippy::cast_possible_wrap,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::if_not_else,
    clippy::inefficient_to_string,
    clippy::items_after_statements,
    clippy::large_digit_groups,
    clippy::large_types_passed_by_value,
    clippy::match_same_arms,
    clippy::missing_const_for_fn,
    clippy::missing_panics_doc,
    clippy::needless_bitwise_bool,
    clippy::needless_collect,
    clippy::needless_pass_by_value,
    clippy::no_effect_underscore_binding,
    clippy::nonstandard_macro_braces,
    clippy::or_fun_call,
    clippy::range_plus_one,
    clippy::range_minus_one,
    clippy::similar_names,
    clippy::suboptimal_flops,
    clippy::too_many_lines,
    clippy::unused_self
)]

use std::{
    env,
    ffi::OsStr,
    fs::{write, File},
    io::Read,
    os,
    path::Path,
    process::{Command, Output},
    str,
};

use anyhow::Result;
#[cfg(feature = "better_panic")]
pub use better_panic;
#[cfg(feature = "pretty_assertions")]
use pretty_assertions::assert_eq;
#[cfg(feature = "regex")]
use regex::Regex;
use tempfile::{tempdir, TempDir};
#[cfg(feature = "better_panic")]
pub mod panic {
    use better_panic::{Settings, Verbosity};

    /// Shortcut to `better_panic::Settings::new().verbosity(better_panic::Verbosity::Minimal).install()`;
    ///
    /// Meant to be used at the start of your tests.
    #[inline]
    pub fn minimal() {
        Settings::new().verbosity(Verbosity::Minimal).install();
    }

    /// Shortcut to `better_panic::Settings::new().verbosity(better_panic::Verbosity::Medium).install()`;
    ///
    /// Meant to be used at the start of your tests.
    #[inline]
    pub fn medium() {
        Settings::new().verbosity(Verbosity::Medium).install();
    }

    /// Shortcut to `better_panic::Settings::new().verbosity(better_panic::Verbosity::Full).install()`;
    ///
    /// Meant to be used at the start of your tests.
    #[inline]
    pub fn full() {
        Settings::new().verbosity(Verbosity::Full).install();
    }
}

#[derive(Debug)]
pub struct Project {
    tempdir: TempDir,
}

/// Shortcut for [`Project::new()`].
#[inline(always)]
pub fn project() -> Result<Project> {
    Project::new()
}

/// Initializes a new sandbox testing environment. Note that **this doesn't initialize a project**, just creates some
/// environment variables with metadata about your project.
///
/// # Panics
///
/// This function may panic if it cannot find the root package metadata (a.k.a your project's metadata).
pub fn init() {
    let md = cargo_metadata::MetadataCommand::new()
        .exec()
        .expect("Couldn't get Cargo Metadata");

    let root = md.root_package().unwrap();
    env::set_var("SANDBOX_TARGET_DIR", &md.target_directory);
    env::set_var("SANDBOX_PKG_NAME", &root.name);
}

impl Project {
    /// Creates a new [`Project`]
    ///
    pub fn new() -> Result<Self> {
        Ok(Self {
            tempdir: tempdir()?,
        })
    }

    /// Gets the [`std::path::Path`] for the [`Project`]'s temporary directory.
    pub fn path(&self) -> &Path {
        self.tempdir.path()
    }

    /// Creates a new file with a relative path to the project's directory.
    ///
    /// `path` gets redirected to the project's real path (temporary and unknown).
    #[inline]
    pub fn new_file<P: AsRef<Path>>(&mut self, path: P, contents: &str) -> Result<()> {
        Ok(write(self.path().join(path), contents)?)
    }

    /// Checks that the contents of a file are correct. It will panic if they aren't, and show the differences if the feature **`pretty_assertions`** is enabled
    ///
    /// `path` gets redirected to the project's real path (temporary and unknown)
    /// # Panics
    /// Will panic if the contents of the file at path aren't encoded as UTF-8
    pub fn check_file<P: AsRef<Path>>(&self, path: P, contents: &str) -> Result<()> {
        let mut f = File::open(self.path().join(path))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let mut buf2 = String::new();
        buf2.push_str(match str::from_utf8(&buf) {
            Ok(val) => val,
            Err(_) => panic!("buf isn't UTF-8 (bug)"),
        });
        assert_eq!(buf2, contents);
        Ok(())
    }

    /// Executes a command relative to the project's directory
    pub fn command<I, S>(&self, args: I) -> Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        #[cfg(feature = "dev")]
        return Ok(Command::new(
            Path::new(&std::env::var("SANDBOX_TARGET_DIR")?)
                .join("debug")
                .join(std::env::var("SANDBOX_PKG_NAME")?),
        )
        .current_dir(self.path())
        .args(args)
        .output()?);

        #[cfg(feature = "release")]
        return Ok(Command::new(
            Path::new(&std::env::var("CARGO_MANIFEST_DIR")?)
                .join("target")
                .join("release")
                .join(env!("CARGO_PKG_NAME")),
        )
        .current_dir(&self.path())
        .args(args)
        .output()?);
    }

    /// Checks the [file signature](https://en.m.wikipedia.org/wiki/File_format#Magic_number) of a file and returns `true` if the file in that path is an executable.
    ///
    /// The checked file signatures are the following, if the file's signature is any of the following, the function will return `true`.
    ///
    /// * `4D 5A`
    ///     - DOS MZ executable and descendants | (.exe, .scr, .sys, .dll, .fon, .cpl, .iec, .ime, .rs, .tsp, .mz)
    /// * `5A 4D`
    ///     - DOS ZM executable and descendants, rare | (.exe)
    /// * `7F 45 4C 46`
    ///     - ELF files
    /// * `64 65 78 0A 30 33 35 00`
    ///     - [Dalvik Executables](https://en.wikipedia.org/wiki/Dalvik_(software))
    /// * `4A 6F 79 21`
    ///     - Preferred Executable Format
    /// * `00 00 03 F3`
    ///     - Amiga Hunk executable file
    pub fn is_bin<P: AsRef<Path>>(&self, path: P) -> bool {
        let mut buf: [u8; 8] = [0; 8];
        let mut f = File::open(self.path().join(&path)).expect("Couldn't open that path");
        match f.read_exact(&mut buf) {
            Ok(()) => {}
            Err(_) => {
                buf.fill(0x01) // Fill the rest of the buffer with 0x01, could have used 0x00 but Dalvik Executable
                               // already ends with 0x00 and that would make false positives
            }
        };

        match buf {
            [0x4D, 0x5A, ..] | [0x5A, 0x4D, ..] | // DOS MZ (.exe, .scr, .sys, .dll, .fon, .cpl, .iec, .ime, .rs, .tsp, .mz)
            [0x7F, 0x45, 0x4C, 0x46, ..] | // ELF
            [0x64, 0x65, 0x78, 0x0A, 0x30, 0x33, 0x35, 0x00] | // Dalvik Executable (.dex)
            [0x4A, 0x6F, 0x79, 0x21, ..] | // Preferred Executable Format
            [0x00, 0x00, 0x03, 0xF3, ..] // Amiga Hunk Executable File
            => {
                true
            }
            _ => {
                false
            }
        }
    }

    /// Creates a [symbolic link](wikipedia.org/wiki/Symlinks), both paths are relative to the temporary project's path.
    ///
    /// # Panics
    ///
    /// This function will panic if the OS can't create a system between the two paths.
    pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(&self, src: P, dst: Q) {
        let src = self.path().join(src.as_ref());
        let dst = self.path().join(dst.as_ref());
        #[cfg(unix)]
        {
            if let Err(e) = os::unix::fs::symlink(&src, &dst) {
                panic!("failed to symlink {:?} to {:?}: {:?}", src, dst, e);
            }
        }
        #[cfg(windows)]
        {
            if src.is_dir() {
                if let Err(e) = os::windows::fs::symlink_dir(&src, &dst) {
                    panic!("failed to symlink {:?} to {:?}: {:?}", src, dst, e);
                }
            } else {
                if let Err(e) = os::windows::fs::symlink_file(&src, &dst) {
                    panic!("failed to symlink {:?} to {:?}: {:?}", src, dst, e);
                }
            }
        }
    }

    /// Cleans your environment used in the working directory (i.e. removing all environment variables that start with a prefix).
    ///
    /// ---
    ///
    /// (Note that "localized" environment variables are taken into account)
    ///
    /// # Panics
    ///
    /// This function will panic if it couldn't get the current working directory, or;
    /// it can't set the current working directory to the temporary project's path, or;
    /// it can't set the current working directory to the original working directory.
    pub fn clean_env(&self, prefix: &str) {
        let cwd = env::current_dir().expect("Couldn't get current working directory");
        // Some environment variables contain some bash script for being defined in X directory (e.g. PROMPT_COMMAND='[[ $PWD == "/foo/bar/" ]] && export FOO=BAR || unset FOO').
        // This will define FOO=BAR only if the current working directory is "/foo/bar".
        // That's why we change the CWD
        env::set_current_dir(self.path()).expect("Couldn't change path");
        for (k, _) in env::vars() {
            if k.starts_with(prefix) {
                env::remove_var(k);
            };
        }

        env::set_current_dir(cwd).expect("Couldn't return to the origin directory");
    }
}

pub trait WithStdout {
    /// Checks that the standard output of a command is what's expected. If they aren't the same, it will show the differences if the `pretty_asssertions` feature is enabled
    ///
    /// ## Example
    /// ```no_run
    /// # use crate::cli_sandbox::WithStdout;
    /// # use std::error::Error;
    /// # use cli_sandbox::project;
    /// # fn main() -> Result<(), Box<dyn Error>>{
    /// let proj = project()?;
    /// let cmd = proj.command(["my", "cool", "--args"])?;
    /// cmd.with_stdout("Expected stdout");
    /// # Ok(())
    /// # }
    /// ```
    fn with_stdout<S: AsRef<str>>(&self, stdout: S);
    /// Checks that the standard error of a command is what's expected. If they aren't the same, it will show the differences if the `pretty_asssertions` feature is enabled
    ///
    /// ## Example
    /// ```no_run
    /// # use std::error::Error;
    /// # use cli_sandbox::{project, WithStdout};
    /// # fn main() -> Result<(), Box<dyn Error>>{
    /// let proj = project()?;
    /// let cmd = proj.command(["my", "cool", "--args"])?;
    /// cmd.with_stderr("Expected stderr");
    /// # Ok(())
    /// # }
    /// ```
    fn with_stderr<S: AsRef<str>>(&self, stderr: S);
    /// Checks that the standard output of a command is what's expected (Using regex). If they aren't the same, it will show the differences if the `pretty_asssertions` feature is enabled
    ///
    /// ## Example
    /// ```no_run
    /// # use std::error::Error;
    /// # use cli_sandbox::{project, WithStdout};
    /// # fn main() -> Result<(), Box<dyn Error>>{
    /// let proj = project()?;
    /// let cmd = proj.command(["my", "cool", "--args"])?;
    /// cmd.with_stdout_regex("<Regex that matches expected stdout>");
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "regex")]
    fn with_stdout_regex<S: AsRef<str>>(&self, stdout: S);
    /// Checks that the standard error of a command is what's expected (Using regex). If they aren't the same, it will show the differences if the `pretty_asssertions` feature is enabled
    ///
    /// ## Example
    /// ```no_run
    /// # use std::error::Error;
    /// # use cli_sandbox::{project, WithStdout};
    /// # fn main() -> Result<(), Box<dyn Error>>{
    /// let proj = project()?;
    /// let cmd = proj.command(["my", "cool", "--args"])?;
    /// cmd.with_stderr("<Regex that matches expected stderr>");
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "regex")]
    fn with_stderr_regex<S: AsRef<str>>(&self, stderr: S);
    /// Returns how many times the program contains the word "warning:" in the `stderr`. Useful for checking compile-time warnings.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// # use cli_sandbox::{project, WithStdout};
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let proj = project()?;
    /// let cmd = proj.command(["my", "cool", "--args"])?;
    /// if cmd.stderr_warns() {
    ///     // Maybe there's something to check with that code...
    /// }
    /// # Ok(())
    /// }
    /// ```
    fn stdout_warns(&self) -> bool;
    /// Returns how many times the program contains the word "warning:" in the `stderr`. Useful for checking compile-time warnings.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// # use cli_sandbox::{project, WithStdout};
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let proj = project()?;
    /// let cmd = proj.command(["my", "cool", "--args"])?;
    /// if cmd.stderr_warns() {
    ///     // Maybe there's something to check with that code...
    /// }
    /// # Ok(())
    /// }
    /// ```
    fn stderr_warns(&self) -> bool;
    /// Checks that the stderr is empty. It's different from `.with_stderr("")` in that this won't print a whole diff. Useful for when ANY presence of a stderr would mean that there were errors, and the output is invalid.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// # use cli_sandbox::{project, WithStdout};
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let proj = project()?;
    /// let cmd = proj.command(["my", "cool", "--args"])?;
    /// if !cmd.empty_stderr() {
    ///     panic!("HELP!!! THE OUTPUT IS INVALID!!");
    /// }
    /// # Ok(())
    /// }
    /// ```
    fn empty_stderr(&self) -> bool;
    /// Checks that the stdout is empty. It's different from `.with_stdout("")` in that this won't print a whole diff. Useful for when ANY presence of a stdout, would mean that there were errors, and the output is invalid.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// # use cli_sandbox::{project, WithStdout};
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let proj = project()?;
    /// let cmd = proj.command(["my", "cool", "--args"])?;
    /// if !cmd.empty_stdout() {
    ///     panic!("HELP!!! THE OUTPUT IS INVALID!!");
    /// }
    /// # Ok(())
    /// }
    /// ```
    fn empty_stdout(&self) -> bool;
    /// Checks that the stdout is corresponding with a file (usually "<my-test>.stdout");
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// # use cli_sandbox::{project, WithStdout};
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let proj = project()?;
    /// let cmd = proj.command(["my", "cool", "--args"])?;
    /// cmd.with_stdout_file("cool-args-test.stdout");
    /// # Ok(())
    /// # }
    /// ```
    fn with_stdout_file<P: AsRef<Path>>(&self, filename: P);
    /// Checks that the stderr is corresponding with a file (usually "<my-test>.stderr");
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// # use cli_sandbox::{project, WithStdout};
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let proj = project()?;
    /// let cmd = proj.command(["my", "cool", "--args"])?;
    /// cmd.with_stderr_file("cool-args-test.stderr");
    /// # Ok(())
    /// # }
    /// ```
    fn with_stderr_file<P: AsRef<Path>>(&self, filename: P);
}

impl WithStdout for Output {
    fn with_stdout<S: AsRef<str>>(&self, stdout: S) {
        let mut buf = String::new();
        buf.push_str(match str::from_utf8(&self.stdout) {
            Ok(val) => val,
            Err(_) => panic!("stdout isn't UTF-8 (bug)"),
        });
        assert_eq!(buf, stdout.as_ref());
    }

    fn with_stderr<S: AsRef<str>>(&self, stderr: S) {
        let mut buf = String::new();
        buf.push_str(match str::from_utf8(&self.stderr) {
            Ok(val) => val,
            Err(_) => panic!("stderr isn't UTF-8 (bug)"),
        });
        assert_eq!(buf, stderr.as_ref());
    }

    #[cfg(feature = "regex")]
    fn with_stderr_regex<S: AsRef<str>>(&self, regex: S) {
        let re = match Regex::new(regex.as_ref()) {
            Ok(re) => re,
            Err(e) => panic!("Regex {} isn't valid: {e}", regex.as_ref()),
        };

        let mut buf = String::new();
        buf.push_str(match str::from_utf8(&self.stderr) {
            Ok(val) => val,
            Err(_) => panic!("stderr isn't UTF-8 (bug)"),
        });

        if !re.is_match(&buf) {
            assert_eq!(buf, regex.as_ref()); // Show differences
        };
    }

    #[cfg(feature = "regex")]
    fn with_stdout_regex<S: AsRef<str>>(&self, regex: S) {
        let re = match Regex::new(regex.as_ref()) {
            Ok(re) => re,
            Err(e) => panic!("Regex {} isn't valid: {e}", regex.as_ref()),
        };

        let mut buf = String::new();
        buf.push_str(match str::from_utf8(&self.stdout) {
            Ok(val) => val,
            Err(_) => panic!("stdout isn't UTF-8 (bug)"),
        });

        if !re.is_match(&buf) {
            assert_eq!(buf, regex.as_ref()); // Show differences
        };
    }

    fn stdout_warns(&self) -> bool {
        let mut buf = String::new();
        buf.push_str(match str::from_utf8(&self.stdout) {
            Ok(val) => val,
            Err(_) => panic!("stdout isn't UTF-8 (bug)"),
        });
        buf.contains("warnings:")
    }

    fn stderr_warns(&self) -> bool {
        let mut buf = String::new();
        buf.push_str(match str::from_utf8(&self.stderr) {
            Ok(val) => val,
            Err(_) => panic!("stderr isn't UTF-8 (bug)"),
        });
        buf.contains("warnings:")
    }

    #[inline]
    fn empty_stderr(&self) -> bool {
        self.stdout.is_empty()
    }

    #[inline]
    fn empty_stdout(&self) -> bool {
        self.stdout.is_empty()
    }

    fn with_stdout_file<P: AsRef<Path>>(&self, filename: P) {
        let expected = match std::fs::read_to_string(&filename) {
            Ok(s) => s,
            Err(e) => panic!("Couldn't read file {}: {e}", filename.as_ref().display()),
        };

        let mut buf = String::new();
        buf.push_str(match str::from_utf8(&self.stdout) {
            Ok(val) => val,
            Err(_) => panic!("stdout isn't UTF-8 (bug)"),
        });

        assert_eq!(expected, buf);
    }

    fn with_stderr_file<P: AsRef<Path>>(&self, filename: P) {
        let expected = match std::fs::read_to_string(&filename) {
            Ok(s) => s,
            Err(e) => panic!("Couldn't read file {}: {e}", filename.as_ref().display()),
        };

        let mut buf = String::new();
        buf.push_str(match str::from_utf8(&self.stderr) {
            Ok(val) => val,
            Err(_) => panic!("stderr isn't UTF-8 (bug)"),
        });

        assert_eq!(expected, buf);
    }
}

#[cfg(feature = "fuzz")]
/// Generates a random string of text, meant to be used a mini-fuzz test. (As input to your CLI.)
///
/// ## Example
///
/// ```no_run
/// # use cli_sandbox::{project, fuzz, WithStdout};
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let proj = project()?;
/// let cmd = proj.command(["name", &fuzz(10)])?; // Use a random string of length 10
/// cmd.with_stdout("...");
/// # Ok(())
/// # }
/// ```
pub fn fuzz(length: usize) -> String {
    let charset = if let Ok(charset) = env::var("CARGO_CFG_FUZZ_CHARSET") {
        charset
    } else {
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890".into()
    };

    let chars = charset.chars().collect::<Vec<char>>();

    let mut buf = String::new();
    for _ in 0..=length {
        buf.push(chars[fastrand::usize(..charset.len())]);
    }

    buf
}

#[cfg(feature = "fuzz_seed")]
/// Generates a random string of text, meant to be used a mini-fuzz test. (As input to your CLI.) It's different from [`fuzz`] because this function also takes a seed, meaining that it will output easily determinitable results.
///
/// ## Example
///
/// ```no_run
/// # use cli_sandbox::{project, fuzz_seed, WithStdout};
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let proj = project()?;
/// let cmd = proj.command(["name", &fuzz_seed(5, 10)])?; // Use a random string of length 10
/// cmd.with_stdout("...");
/// # Ok(())
/// # }
/// ```
pub fn fuzz_seed(length: usize, seed: u64) -> String {
    fastrand::seed(seed);
    let charset = if let Ok(charset) = env::var("CARGO_CFG_FUZZ_CHARSET") {
        charset
    } else {
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890".into()
    };

    let mut chars = charset.chars();

    let mut buf = String::new();
    for _ in 0..=length {
        buf.push(
            chars
                .nth(fastrand::u8(..charset.len() as u8).into())
                .unwrap(),
        );
    }

    charset
}

pub const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
