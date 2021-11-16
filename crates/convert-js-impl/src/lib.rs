// Copyright (c) 2021 René Kijewski <rene.[SURNAME]@fu-berlin.de>
// All rights reserved.
//
// This software and the accompanying materials are made available under
// the terms of the ISC License which is available in the project root as LICENSE-ISC, AND/OR
// the terms of the MIT License which is available at in the project root as LICENSE-MIT, AND/OR
// the terms of the Apache License, Version 2.0 which is available in the project root as LICENSE-APACHE.
//
// You have to accept AT LEAST one of the aforementioned licenses to use, copy, modify, and/or distribute this software.
// At your will you may redistribute the software under the terms of only one, two, or all three of the aforementioned licenses.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! TODO: doc

use std::env::var;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command, Stdio};

use is_executable::is_executable;
use tempfile::TempDir;
use thiserror::Error;
use which::which;

mod private {
    #[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Unit();
}

/// TODO: doc
#[derive(Debug, Copy, Clone)]
pub enum Exit {
    /// TODO: doc
    Code(i32),
    /// TODO: doc
    Signal(i32),
    /// TODO: doc
    Unknown,
}

/// TODO: doc
#[derive(Debug, Error)]
pub enum Error {
    /// TODO: doc
    #[error("yarn needs to be installed")]
    YarnNotFound {
        /// TODO: doc
        #[source]
        source: which::Error,
    },

    /// TODO: doc
    #[error("Tried to execute {cmd:?}, but it returned an error")]
    CouldNotStart {
        /// TODO: doc
        #[source]
        source: std::io::Error,
        /// TODO: doc
        cmd: Box<Command>,
    },

    /// TODO: doc
    #[error(r#"IO error while reading converting file"#)]
    IoError {
        /// TODO: doc
        #[source]
        source: std::io::Error,
    },

    /// TODO: doc
    #[error("Tried to execute {cmd:?}, but it had an error: {exit:?}")]
    ExecError {
        /// TODO: doc
        exit: Exit,
        /// TODO: doc
        cmd: Box<Command>,
    },

    /// TODO: doc
    #[error(r#"Crate "convert-js-impl" is located in a non-UTF-8 path"#)]
    NonUTF8Installation,

    #[doc(hidden)]
    #[error("...")]
    _NonExhausive(private::Unit),
}

/// TODO: doc
pub type Result<T = String, E = Error> = std::result::Result<T, E>;

const COMPRESS_OPTIONS: &str = "\
drop_console=false,\
drop_debugger=false,\
hoist_funs=true,\
hoist_vars=true,\
passes=3,\
pure_getters=true,\
sequences=false,\
unsafe_comps=true,\
unsafe_math=true,\
unsafe_proto=true,\
unsafe_undefined=true\
";

const BEAUTIFY_OPTIONS: &str = "\
ascii_only=true,\
beautify=false,\
inline_script=true,\
semicolons=true,\
webkit=true\
";

fn run(mut cmd: Command) -> Result<()> {
    match cmd.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => {
            let cmd = cmd.into();
            let exit = match (status.signal(), status.code()) {
                (Some(signal), _) if signal != 0 => Exit::Signal(signal),
                (_, Some(code)) if code != 0 => Exit::Code(code),
                _ => Exit::Unknown,
            };
            Err(Error::ExecError { cmd, exit })
        }
        Err(source) => {
            let cmd = cmd.into();
            Err(Error::CouldNotStart { cmd, source })
        }
    }
}

/// TODO: doc
pub fn convert_js(js_code: impl AsRef<[u8]>) -> Result<String> {
    let js_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("js-src");

    let babel_cfg = js_dir.join("babel.config.json");
    let babel_cfg = babel_cfg.to_str().ok_or(Error::NonUTF8Installation)?;

    let node_modules = js_dir.join("node_modules/.bin");
    let babel = node_modules.join("babel");
    let browserslist = node_modules.join("browserslist");
    let uglifyjs = node_modules.join("uglifyjs");

    let yarn = which("yarn").map_err(|source| Error::YarnNotFound { source })?;

    if !is_executable(&babel) || !is_executable(&browserslist) || !is_executable(&uglifyjs) {
        let mut cmd = Command::new(&yarn);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.current_dir(&js_dir);
        run(cmd)?;
    }

    // TODO: do daily
    let mut cmd = Command::new(browserslist);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.current_dir(&js_dir);
    cmd.arg("--update-db");
    run(cmd)?;

    let tempdir = match var("CARGO_TARGET_TMPDIR") {
        Ok(tempdir) => TempDir::new_in(tempdir),
        Err(_) => TempDir::new(),
    };
    let tempdir = tempdir.expect("Could not create temp dir");

    // write input JS file
    let mut file = OpenOptions::new()
        .create(true)
        .create_new(true)
        .write(true)
        .open(tempdir.path().join("input.js"))
        .expect(r#"Could not create temp file "input.js""#);
    file.write_all(js_code.as_ref())
        .expect(r#"Could not write JS code to temp "input.js""#);
    file.sync_all()
        .expect(r#"Could not sync temp file "input.js""#);
    drop(file);

    // convert to ES5
    let mut cmd = Command::new(babel);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.current_dir(tempdir.path());
    cmd.arg(format!("--config-file={}", babel_cfg));
    cmd.arg("--out-file=es5.js");
    cmd.arg("--").arg("input.js");
    run(cmd)?;

    // minify JS code
    let mut cmd = Command::new(uglifyjs);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.current_dir(tempdir.path());
    cmd.arg("--compress").arg(COMPRESS_OPTIONS);
    cmd.arg("--beautify").arg(BEAUTIFY_OPTIONS);
    cmd.arg("--mangle");
    cmd.arg("--validate");
    cmd.arg("--output").arg("min.js");
    cmd.arg("--").arg("es5.js");
    run(cmd)?;

    // read generated file
    let mut content = String::new();
    OpenOptions::new()
        .read(true)
        .open(tempdir.path().join("min.js"))
        .map_err(|source| Error::IoError { source })?
        .read_to_string(&mut content)
        .map_err(|source| Error::IoError { source })?;

    Ok(content)
}
