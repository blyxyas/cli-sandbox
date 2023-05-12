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
    path::Path,
    process::{Command, Output},
    str,
};

use anyhow::Result;
#[cfg(feature = "pretty_assertions")]
use pretty_assertions::assert_eq;
#[cfg(feature = "regex")]
use regex::Regex;
use tempfile::{tempdir, TempDir};

#[derive(Debug)]
pub struct Project {
    tempdir: TempDir,
}

/// Shortcut for [`Project::new()`].
#[inline(always)]
pub fn project() -> Result<Project> {
    Project::new()
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
            Path::new(&std::env::var("CARGO_TARGET_DIR")?)
                .join("debug")
                .join(env!("CARGO_PKG_NAME")),
        )
        .current_dir(self.path())
        .args(args)
        .output()?);

        #[cfg(feature = "release")]
        return Ok(Command::new(
            Path::new(&std::env::var("CARGO_TARGET_DIR")?)
                .join("release")
                .join(env!("CARGO_PKG_NAME")),
        )
        .current_dir(&self.path())
        .args(args)
        .output()?);
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
        self.stdout.len() == 0
    }

    #[inline]
    fn empty_stdout(&self) -> bool {
        self.stdout.len() == 0
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
