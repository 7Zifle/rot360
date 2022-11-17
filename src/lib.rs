use clap::Parser;
use glob::glob;
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::io;
use std::process::Command;
use std::thread;
use std::time::Duration;



#[derive(PartialEq)]
pub enum Backend {
    Sway,
    Xorg,
}

// A program to automatically switch display when rotating.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    // Runs the program without the loop
    #[arg(short, long)]
    oneshot: bool,
    // time to sleep inbetween loops
    #[arg(short, long, default_value_t = 1000)]
    sleep: u64,
    // display string for the display that will flip
    #[arg(short, long, default_value_t=String::from("eDP-1"))]
    display: String,
    // list of touchscreen devices to bind to display
    #[arg(long)]
    touchscreen: Vec<String>,
    // the threshold for orientation
    #[arg(short, long, default_value_t=0.2)]
    threshold: f32,
    // Factor to normalize axis' value
    #[arg(long, default_value_t=1000000.0)]
    normalization_factor: f32,
    // option to disable keyboard
    #[arg(long)]
    keyboard: bool,
}

fn get_keyboards(backend: &Backend) -> Result<Vec<String>, String> {
    match backend {
        Backend::Sway => {
            let raw_inputs = String::from_utf8(
                Command::new("swaymsg")
                    .arg("-t")
                    .arg("get_inputs")
                    .arg("--raw")
                    .output()
                    .expect("Swaymsg get inputs command failed")
                    .stdout,
            )
            .unwrap();

            let mut keyboards = vec![];
            let deserialized: Vec<Value> = serde_json::from_str(&raw_inputs)
                .expect("Unable to deserialize swaymsg JSON output");
            for output in deserialized {
                let input_type = output["type"].as_str().unwrap();
                if input_type == "keyboard" {
                    keyboards.push(output["identifier"].to_string());
                }
            }

            Ok(keyboards)
        }
        Backend::Xorg => Ok(vec![]),
    }
}

fn detect_backend() -> Result<Backend, String> {
    let sway_output = String::from_utf8(Command::new("pidof").arg("sway").output().unwrap().stdout);
    let xorg_output = String::from_utf8(Command::new("pidof").arg("Xorg").output().unwrap().stdout);
    let x_output = String::from_utf8(Command::new("pidof").arg("X").output().unwrap().stdout);

    if !sway_output.unwrap().is_empty() {
        Ok(Backend::Sway)
    } else if !xorg_output.unwrap().is_empty() || !x_output.unwrap().is_empty() {
        Ok(Backend::Xorg)
    } else {
        Err("Unable to find Sway or Xorg procceses".to_owned())
    }
}


#[derive(Copy, Clone)]
struct Orientation {
    vector: (f32, f32),
    sway_state: &'static str,
}

impl Orientation {
    fn search(accelometers: &Vec<Accelerometer>, threshold: f32, normalizaiton: f32) -> Orientation {
        let orientations = [
            Orientation {
                vector: (0.0, -1.0),
                sway_state: "normal",
            },
            Orientation {
                vector: (0.0, 1.0),
                sway_state: "180",
            },
            Orientation {
                vector: (-1.0, 0.0),
                sway_state: "90",
            },
            Orientation {
                vector: (1.0, 0.0),
                sway_state: "270",
            },
        ];

	// find the best orienation for one censor
	let x = accelometers[0].x as f32 / normalizaiton;
	let y = accelometers[0].y as f32 / normalizaiton;
	println!("x: {} y: {}", x, y);
	let mut current = orientations[0];
	for orient in orientations.iter() {
            let d = (x - orient.vector.0).powf(2.0) + (y - orient.vector.1).powf(2.0);

            if d < threshold {
		println!("found the best: {}", orient.sway_state);
		current = orient.clone();
		break;
            }
        }
        /*
        let or1 = [
            // this closed.
            0, 0, 1, // iio:device1
            0, 0, -1, // iio: device3
p        ];
        let or2 = [
            // this is half open 45deg most used
            0, -1, 0, // iio:device1
            0, 0, -1, // iio:device3
        ];
        let or3 = [
            // this is open 90deg
            0, 0, -1, // iio:device1
            0, 0, -1, // iio:device3
        ];
        */
	current
    }
}

pub struct Accelerometer {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Accelerometer {
    pub fn build() -> Result<Vec<Accelerometer>, Box<dyn Error>> {
	let accel_paths = find_accelerometers()?;
        // load device 1
        let dev1_x = fs::read_to_string(&accel_paths[0])?;
        let dev1_x = dev1_x.trim_end_matches("\n").parse::<i32>()?;

        let dev1_y = fs::read_to_string(&accel_paths[1])?;
        let dev1_y = dev1_y.trim_end_matches("\n").parse::<i32>()?;

        let dev1_z = fs::read_to_string(&accel_paths[2])?;
        let dev1_z = dev1_z.trim_end_matches("\n").parse::<i32>()?;

        // load device 3
        let dev3_x = fs::read_to_string(&accel_paths[3])?;
        let dev3_x = dev3_x.trim_end_matches("\n").parse::<i32>()?;

        let dev3_y = fs::read_to_string(&accel_paths[4])?;
        let dev3_y = dev3_y.trim_end_matches("\n").parse::<i32>()?;

        let dev3_z = fs::read_to_string(&accel_paths[5])?;
        let dev3_z = dev3_z.trim_end_matches("\n").parse::<i32>()?;

        Ok(vec![
            Accelerometer {
                x: dev1_x,
                y: dev1_y,
                z: dev1_z,
            },
            Accelerometer {
                x: dev3_x,
                y: dev3_y,
                z: dev3_z,
            },
        ])
    }

    pub fn run(config: &Config) -> Result<(), String> {
        let backend = detect_backend()?;
        
	let mut saved_state = "";
        loop {
            let accelerometers = match Accelerometer::build() {
                Ok(acclers) => acclers,
                Err(e) => return Err(e.to_string()),
            };
	    // TODO: check orientation
	    let orientation = Orientation::search(&accelerometers, config.threshold, config.normalization_factor);
	    println!("state: {}", orientation.sway_state);
	    if orientation.sway_state != saved_state {
		update_sway(&backend, &config.display, orientation.sway_state)?;
		
		if config.keyboard {
		    update_keyboards(&backend, orientation.sway_state)?;
		}
		
		if config.oneshot {
		    return Ok(());
		}
		saved_state = orientation.sway_state;
	    }
	    thread::sleep(Duration::from_millis(config.sleep));
        }
    }
}

fn update_keyboards(backend: &Backend, state: &str) -> Result<(), String> {
    let keyboards = get_keyboards(&backend)?;
    let keyboard_state = if state == "normal" {
        "enable"
    } else {
        "disable"
    };
    for keyboard in &keyboards {
        //                            println!("swaymsg input {} events {}", keyboard, keyboard_state);
        Command::new("swaymsg")
            .arg("input")
            .arg(keyboard)
            .arg("events")
            .arg(keyboard_state)
            .spawn()
            .expect("Swaymsg keyboard command failed to start")
            .wait()
            .expect("Swaymsg keyboard command wait failed");
    }
    Ok(())
}

fn find_accelerometers() -> io::Result<Vec<String>> {
    let mut paths = Vec::<String>::new();

    for entry in glob("/sys/bus/iio/devices/iio:device*/in_accel_*_raw").unwrap() {
	match entry {
	    Ok(path) => {
		paths.push(String::from(path.to_str().unwrap()));
	    },
	    Err(e) => println!("{:?}", e),
	}
    }
    Ok(paths)
}

fn update_sway(backend: &Backend, display: &str, new_state: &str) -> Result<(), &'static str> {
    if backend != &Backend::Sway {
        return Err("Backend is not sway");
    }
    Command::new("swaymsg")
        .arg("output")
        .arg(display)
        .arg("transform")
        .arg(new_state)
        .spawn()
        .expect("Swaymsg rotate command failed to start")
        .wait()
        .expect("Swaymsg rotate command wait failed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn read_accelerometers() {
	let paths = find_accelerometers();
	assert!(paths.is_ok());
	let paths = paths.unwrap();
	assert_eq!(paths.len(), 6);
	for path in paths {
	    println!("path: {}", path);
	}
    }
}
