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
    pub fn new_file<P: AsRef<Path>>(&mut self, path: P, contents: &str) -> Result<()> {
        write(self.path().join(path), contents)?;
        Ok(())
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
	/// Checks that the standard output of a command is what's expected.
    fn with_stdout<S: AsRef<str>>(&self, stdout: S);
    fn with_stderr<S: AsRef<str>>(&self, stderr: S);
	fn stdout_warns(&self) -> bool;
	fn stderr_warns(&self) -> bool;
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

	/// Returns how many times the program contains the word "warning:" in the `stderr`. Useful for checking compile-time warnings.
	/// 
	/// ## Example
	/// 
	/// ```
	/// # fn main() -> Result<(), Box<dyn Error>> {
	/// let cmd = project()?;
	/// if cmd.stderr_warns() {
	/// 	// Maybe there's something to check with that code...
	/// }
	/// # Ok(())
	/// }
	/// ```
	fn stdout_warns(&self) -> bool {
		let mut buf = String::new();
		buf.push_str(match str::from_utf8(&self.stdout) {
			Ok(val) => val,
			Err(_) => panic!("stdout isn't UTF-8 (bug)"),
		});
		buf.contains("warnings:")
	}

	/// Returns how many times the program contains the word "warning:" in the `stderr`. Useful for checking compile-time warnings.
	/// 
	/// ## Example
	/// 
	/// ```
	/// # fn main() -> Result<(), Box<dyn Error>> {
	/// let cmd = project()?;
	/// if cmd.stderr_warns() {
	/// 	// Maybe there's something to check with that code...
	/// }
	/// # Ok(())
	/// }
	/// ```
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
