#![windows_subsystem = "windows"]


#[macro_use]
extern crate clap;


use std::path::*;

mod rim;
use rim::app::App;

use rim::util::*;

fn main() {
    let matches = clap_app!(myapp =>
        (version: "0.0.4")
        (author: "Nimaoth")
        (about: "View images")
        (@arg file: +required +takes_value "Display this file or files in this directory")
        (@arg floating: -f --float "Open as floating window")
        (@arg size: -s --size +takes_value +multiple #{2, 2} "Size of floating window")
    )
    .get_matches();

    let mut floating = false;
    let (mut width, mut height) = (1000, 900);
    if matches.is_present("floating") {
        floating = true;
    }
    match matches.values_of("size") {
        Some(values) => {
            let values: Vec<_> = values.collect();
            width = values[0].parse().expect("Width must be a number");
            height = values[1].parse().expect("Height must be a number");
        },
        None => {},
    };

    let mut app = App::new(floating, width, height);

    let path = get_absolute_path(&PathBuf::from(matches.value_of("file").unwrap()));
    if Path::is_file(&path) {
        match app.open_image(Path::new(&path), false) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Failed to load image {:?}", path);
            },
        }
    } else if Path::is_dir(&path) {
        match std::fs::read_dir(path) {
            Ok(dir) => {
                for image_path in dir.into_iter() {
                    match image_path {
                        Ok(path) => {
                            let path = get_absolute_path(&path.path());
                            match app.open_image(&path, false) {
                                Ok(_) => {},
                                Err(_) => eprintln!("Failed to load image '{:?}'", &path),
                            }
                        }
                        Err(msg) => eprintln!("Error: {}", msg),
                    }
                }
            }
            Err(msg) => eprintln!("Failed to load files in directory: {}", msg),
        }
    } else {
        eprintln!("path is not a file or directory: {:?}", path);
        return;
    }

    app.run();
}
