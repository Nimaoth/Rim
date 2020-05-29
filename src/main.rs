#[macro_use]
extern crate clap;

use std::path::*;

mod rim;
use rim::app::App;

fn main() {
    let matches = clap_app!(myapp =>
        (version: "0.0.3")
        (author: "Nimaoth")
        (about: "View images")
        (@arg file: -f +takes_value "Display this file")
        (@arg directory: -d +takes_value "Display files in this folder")
    )
    .get_matches();

    let mut app = App::new();

    if matches.is_present("file") {
        let image_path = matches.value_of("file").unwrap();
        match app.open_image(Path::new(image_path)) {
            Ok(_) => {},
            Err(_) => eprintln!("Failed to load image '{}'", image_path),
        }
    }
    if matches.is_present("directory") {
        let dir_path = matches.value_of("directory").unwrap();
        match std::fs::read_dir(dir_path) {
            Ok(dir) => {
                for image_path in dir.into_iter() {
                    match image_path {
                        Ok(path) => {
                            let path = path.path().canonicalize().unwrap();
                            match app.open_image(&path) {
                                Ok(_) => {},
                                Err(_) => eprintln!("Failed to load image '{:?}'", &path),
                            }
                        }
                        Err(msg) => eprintln!("Error: {}", msg),
                    }
                }
            }
            Err(msg) => eprintln!("Failed to load files in directory '{}': {}", dir_path, msg),
        }
    }

    app.run();
}
