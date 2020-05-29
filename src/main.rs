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
        (@arg file: +required +takes_value "Display this file or files in this directory")
    )
    .get_matches();

    let mut app = App::new();

    let path = PathBuf::from(matches.value_of("file").unwrap()).canonicalize().expect("File doesn't exist");
    if Path::is_file(&path) {
        match app.open_image(Path::new(&path)) {
            Ok(_) => {},
            Err(_) => eprintln!("Failed to load image {:?}", path),
        }
    } else if Path::is_dir(&path) {
        match std::fs::read_dir(path) {
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
            Err(msg) => eprintln!("Failed to load files in directory: {}", msg),
        }
    } else {
        eprintln!("path is not a file or directory: {:?}", path);
        return;
    }

    app.run();
}
