//!---
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::prelude::*;

mod error;
mod prelude;
mod utils;

slint::include_modules!();

#[expect(clippy::unnecessary_wraps)]
fn main() -> Result<()> {
    Ok(())
}
