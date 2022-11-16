use std::fs;
use std::process::Command;

use clap::Parser;
use rot8::{Backend, Accelerometer, Config};


fn main() -> Result<(), String> {
    let config = Config::parse();
    Accelerometer::run(&config)?;
    Ok(())
}
