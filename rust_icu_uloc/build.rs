// Copyright 2020 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! See LICENSE for licensing information.
//!
//! This build.rs script provides Cargo _features_ indicating the target ICU4C library version,
//! enabling some conditionally compiled Rust code in this crate that depends on the particular
//! ICU4C version.
//!
//! Please refer to README.md for instructions on how to build the library for your use.

use {
    anyhow::{Context, Result},
    std::process,
};

/// A `Command` that also knows its name.
struct Command {
    name: String,
    rep: process::Command,
}

impl Command {
    /// Creates a new command to run, with the executable `name`.
    pub fn new(name: &'static str) -> Self {
        let rep = process::Command::new(&name);
        let name = String::from(name);
        Command { name, rep }
    }

    /// Runs this command with `args` as arguments.
    pub fn run(&mut self, args: &[&str]) -> Result<String> {
        self.rep.args(args);
        let stdout = self.stdout()?;
        Ok(String::from(&stdout).trim().to_string())
    }

    // Captures the stdout of the command.
    fn stdout(&mut self) -> Result<String> {
        let output = self
            .rep
            .output()
            .with_context(|| format!("could not execute command: {}", self.name))?;
        let result = String::from_utf8(output.stdout)
            .with_context(|| format!("could not convert output to UTF8"))?;
        Ok(result.trim().to_string())
    }
}

/// A command representing an auto-configuration detector.  Use `ICUConfig::new()` to create.
struct ICUConfig {
    rep: Command,
}

impl ICUConfig {
    /// Creates a new ICUConfig.
    fn new() -> Self {
        ICUConfig {
            rep: Command::new("pkg-config"),
        }
    }
    /// Obtains the major-minor version number for the library. Returns a string like `64.2`.
    fn version(&mut self) -> Result<String> {
        self.rep
            .run(&["--modversion", "icu-i18n"])
            .with_context(|| format!("while getting ICU version; is icu-config in $PATH?"))
    }

    /// Returns the config major number.  For example, will return "64" for
    /// version "64.2"
    fn version_major() -> Result<String> {
        let version = ICUConfig::new().version()?;
        let components = version.split(".");
        let last = components
            .take(1)
            .last()
            .with_context(|| format!("could not parse version number: {}", version))?;
        Ok(last.to_string())
    }

    fn version_major_int() -> Result<i32> {
        let version_str = ICUConfig::version_major()?;
        Ok(version_str.parse().unwrap())
    }
}

fn main() -> Result<()> {
    std::env::set_var("RUST_BACKTRACE", "full");
    let icu_major_version = ICUConfig::version_major_int()?;
    println!("icu-major-version: {}", icu_major_version);
    if icu_major_version >= 64 {
        println!("cargo:rustc-cfg=feature=\"icu_version_64_plus\"");
    }
    if icu_major_version >= 67 {
        println!("cargo:rustc-cfg=feature=\"icu_version_67_plus\"");
    }
    println!("done");
    Ok(())
}
