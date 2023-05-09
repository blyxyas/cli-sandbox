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
    clippy::missing_assert_message,
    clippy::semicolon_outside_block,
    clippy::str_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::unneeded_field_pattern,
    clippy::use_debug
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
#[cfg(feature = "unstable")]
use stability::unstable;
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
    /// If the `regex` feature is enabled, you could write the expected output as a regex (being more flexible)
    ///
    /// ## Example
    /// ```rust, ignore
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
    /// Checks that the standard output of a command is what's expected. If they aren't the same, it will show the differences if the `pretty_asssertions` feature is enabled
    ///
    /// If the `regex` feature is enabled, you could write the expected output as a regex (being more flexible)
    ///
    /// ## Example
    /// ```rust, ignore
    /// # use std::error::Error;
    /// # use cli_sandbox::{project, WithStdout};
    /// # fn main() -> Result<(), Box<dyn Error>>{
    /// let proj = project()?;
    /// let cmd = proj.command(["my", "cool", "--args"])?;
    /// cmd.with_stderr("Expected stdout");
    /// # Ok(())
    /// # }
    /// ```
    fn with_stderr<S: AsRef<str>>(&self, stderr: S);
    /// Returns how many times the program contains the word "warning:" in the `stderr`. Useful for checking compile-time warnings.
    ///
    /// ## Example
    ///
    /// ```rust, ignore
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
    /// ```rust, ignore
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
}

impl WithStdout for Output {
    #[cfg(not(feature = "regex"))]
    fn with_stdout<S: AsRef<str>>(&self, stdout: S) {
        let mut buf = String::new();
        buf.push_str(match str::from_utf8(&self.stdout) {
            Ok(val) => val,
            Err(_) => panic!("stdout isn't UTF-8 (bug)"),
        });
        assert_eq!(buf, stdout.as_ref());
    }

    #[cfg(not(feature = "regex"))]
    fn with_stderr<S: AsRef<str>>(&self, stderr: S) {
        let mut buf = String::new();
        buf.push_str(match str::from_utf8(&self.stderr) {
            Ok(val) => val,
            Err(_) => panic!("stderr isn't UTF-8 (bug)"),
        });
        assert_eq!(buf, stderr.as_ref());
    }

    #[cfg(feature = "regex")]
    fn with_stderr<S: AsRef<str>>(&self, regex: S) {
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
    fn with_stdout<S: AsRef<str>>(&self, regex: S) {
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
}

pub const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
