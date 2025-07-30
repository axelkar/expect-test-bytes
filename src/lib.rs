//! Copy of [expect-test](https://github.com/rust-analyzer/expect-test), a minimalistic snapshot
//! testing library, for bytes and binary data.
//!
//! # Example
//!
//! ```
//! let actual = b"example\n";
//!
//! expect_test_bytes::expect_file!["test_data/example"].assert_eq(actual);
//! ```

use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{fmt, fs, io};

const UPDATE_EXPECT_VAR_NAME: &str = if cfg!(test) {
    "UPDATE_EXPECT_BYTES"
} else {
    "UPDATE_EXPECT"
};

const HELP: &str = "
You can update all `expect!` tests by running:

    env UPDATE_EXPECT=1 cargo test

To update a single test, place the cursor on `expect` token and use `run` feature of rust-analyzer.
";

static HELP_PRINTED: AtomicBool = AtomicBool::new(false);

/// Converts `ErrorKind::NotFound` to `Ok(None)`
fn not_found_to_none<T>(res: io::Result<T>) -> io::Result<Option<T>> {
    match res {
        Ok(value) => Ok(Some(value)),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// Finds the first index where the elements of `a` and `b` differ.
///
/// If the elements don't differ but the number of elements differ, the first index where only one
/// slice has an element is returned.
fn first_diff_index(a: &[u8], b: &[u8]) -> Option<usize> {
    a.iter()
        .zip(b.iter())
        .position(|(x, y)| x != y)
        .or_else(|| (a.len() != b.len()).then(|| a.len().min(b.len())))
}

const BYTE_WINDOW_HALF_SIZE: usize = 4;

struct ByteWindowDisplay<'a> {
    data: &'a [u8],
    diff_idx: usize,
    is_expected: bool,
}
impl fmt::Display for ByteWindowDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start = self.diff_idx.saturating_sub(BYTE_WINDOW_HALF_SIZE);
        let end = self.data.len().min(self.diff_idx + BYTE_WINDOW_HALF_SIZE);

        // same as `self.diff_idx.min(BYTE_WINDOW_HALF_SIZE)`
        let translated_diff_idx = self.diff_idx - start;

        for (i, byte) in self.data[start..=end].iter().enumerate() {
            if i != 0 {
                write!(f, " ").unwrap();
            }
            if i == translated_diff_idx {
                let highlight_ansi_code = if self.is_expected { "32" } else { "31" };
                write!(f, "\x1b[{highlight_ansi_code}m").unwrap();
            }

            write!(f, "{byte:02x}").unwrap();

            if i == translated_diff_idx {
                write!(f, "\x1b[0m").unwrap();
            }
        }

        write!(f, " {}", CharacterPanel(&self.data[start..=end]))?;
        Ok(())
    }
}

/// <https://github.com/sharkdp/hexyl/blob/9ef7c346dda6320bb5d746810b9e93e1a66e7fc0/src/lib.rs#L30-L32>
struct CharacterPanel<'a>(&'a [u8]);
impl fmt::Display for CharacterPanel<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            let ch = match *byte {
                0 => '⋄',
                _ if byte.is_ascii_graphic() => *byte as char,
                b' ' => ' ',
                _ if byte.is_ascii_whitespace() => '_',
                _ if byte.is_ascii() => '•',
                _ => '×',
            };
            f.write_char(ch)?;
        }
        Ok(())
    }
}

/// Self-updating file.
///
/// [`ExpectFile::assert_eq`] updates the file when the `UPDATE_EXPECT` environment variable is
/// set.
#[derive(Debug)]
pub struct ExpectFile {
    #[doc(hidden)]
    pub path: PathBuf,
}

impl ExpectFile {
    /// Checks whether file's contents are equal to `actual`.
    ///
    /// When the `UPDATE_EXPECT` environment variable is set, the file is updated or created with
    /// the data from `actual`.
    ///
    /// # Panics
    ///
    /// Will panic when the file's contents don't equal `actual` and `UPDATE_EXPECT` is not set or
    /// if writing to stdout or updating the file fails.
    pub fn assert_eq(&self, actual: &[u8]) {
        if let Err(()) = self.assert_eq_nopanic_imp(actual, &mut io::stdout()) {
            // Use resume_unwind instead of panic!() to prevent a backtrace, which is unnecessary noise.
            std::panic::resume_unwind(Box::new(()));
        }
    }
    fn assert_eq_nopanic_imp<W: io::Write>(&self, actual: &[u8], writer: &mut W) -> Result<(), ()> {
        let expected = not_found_to_none(fs::read(&self.path)).unwrap();
        if expected.as_deref() == Some(actual) {
            return Ok(());
        }
        if std::env::var_os(UPDATE_EXPECT_VAR_NAME).is_some() {
            writeln!(
                writer,
                "\x1b[1m\x1b[92mupdating\x1b[0m: {}",
                self.path.display()
            )
            .unwrap();
            fs::write(&self.path, actual).unwrap();
            return Ok(());
        }
        let print_help = if cfg!(test) {
            true // Tests are run in the same process in arbitrary order
        } else {
            !HELP_PRINTED.swap(true, Ordering::SeqCst)
        };
        let help = if print_help { HELP } else { "" };

        writeln!(
            writer,
            "
\x1b[1m\x1b[91merror\x1b[97m: expect test failed\x1b[0m
   \x1b[1m\x1b[34m-->\x1b[0m {location}
{help}
\x1b[1mExpect\x1b[0m:
{expect}

\x1b[1mActual\x1b[0m:
<binary>
",
            location = self.path.display(),
            expect = if expected.is_some() {
                "<binary>"
            } else {
                "\x1b[1mNot found\x1b[0m"
            },
        )
        .unwrap();

        if let Some(expected) = expected {
            let diff_idx = first_diff_index(&expected, actual).unwrap_or(0);

            writeln!(
                writer,
                "\x1b[1mDiff\x1b[0m:
Binary files differ at byte {diff_idx:#x}

Expect: {expect}
Actual: {actual}
        {offset}\x1b[1m^^\x1b[0m",
                expect = ByteWindowDisplay {
                    data: &expected,
                    diff_idx,
                    is_expected: true
                },
                actual = ByteWindowDisplay {
                    data: actual,
                    diff_idx,
                    is_expected: false
                },
                offset = "   ".repeat(diff_idx.min(BYTE_WINDOW_HALF_SIZE)),
            )
            .unwrap();
        }

        Err(())
    }
}

/// Creates an instance of [`ExpectFile`] from a relative or absolute path:
///
/// ```
/// # use expect_test_bytes::expect_file;
/// expect_file!["test_data/example"];
/// ```
#[macro_export]
macro_rules! expect_file {
    [$path:expr] => {
        $crate::ExpectFile {
            path: {
                let path = ::std::path::Path::new($path);
                if path.is_absolute() {
                    path.to_owned()
                } else {
                    ::std::path::Path::new(file!()).parent().unwrap().join(path)
                }
            },
        }
    };
}

/// Bytes.
///
/// Self-updating hasn't been implemented yet.
#[derive(Debug)]
pub struct Expect<'a> {
    #[doc(hidden)]
    pub position: Position,
    #[doc(hidden)]
    pub data: &'a [u8],
}

/// Position of original `expect!` in the source file.
#[derive(Debug)]
pub struct Position {
    #[doc(hidden)]
    pub file: &'static str,
    #[doc(hidden)]
    pub line: u32,
    #[doc(hidden)]
    pub column: u32,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// Creates an instance of [`Expect`] from an expression returning bytes:
///
/// ```
/// # use expect_test_bytes::expect;
/// expect![[b"example"]];
/// ```
#[macro_export]
macro_rules! expect {
    [[$data:expr]] => {
        $crate::Expect {
            position: $crate::Position {
                file: file!(),
                line: line!(),
                column: column!(),
            },
            data: $data,
        }
    };
    [$data:expr] => { $crate::expect![[$data]] };
    [] => { $crate::expect![[b""]] };
    [[]] => { $crate::expect![[b""]] };
}

#[cfg(test)]
mod tests;
