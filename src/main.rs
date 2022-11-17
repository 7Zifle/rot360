use clap::Parser;
use rot360::{Accelerometer, Config};

fn main() -> Result<(), String> {
    let config = Config::parse();
    Accelerometer::run(&config)?;
    Ok(())
}
