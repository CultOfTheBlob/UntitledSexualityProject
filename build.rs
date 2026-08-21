//! Builds all slint ui components into rust code.
#![allow(clippy::expect_used)]

use slint_build::CompilerConfiguration;

fn main() {
    let config = CompilerConfiguration::new().with_style(String::from("cosmic-dark"));

    slint_build::compile_with_config("ui/main-window.slint", config).expect("Slint build failed");
}
