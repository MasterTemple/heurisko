use std::path::Path;

use walkdir::WalkDir;

use crate::{
    app_config::APP_EXT,
    hsk_file::{HskFile, HskResult},
    CONFIG,
};

pub fn command_convert(
    source: String,
    destination: Option<String>,
    flatten: bool,
) -> HskResult<()> {
    let mut data_dir = CONFIG.data_dir();
    let source = Path::new(&source);
    let dest = destination.unwrap_or(String::new());
    let dest = Path::new(&dest);
    data_dir.push(dest);

    if source.is_file() {
        print!("File: ");
        let mut dest = data_dir.join(source.file_name().unwrap());
        dest.set_extension(APP_EXT);
        println!("Converting: {source:?} -> {dest:?}\n");
        HskFile::convert(source, dest.as_path())?;
    }
    if source.is_dir() {
        println!("Directory:");
        let walker = WalkDir::new(source);
        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let mut dest = data_dir.clone();
                if flatten {
                    dest.push(path.file_name().unwrap());
                } else {
                    dest.push(path.strip_prefix(source).unwrap());
                }
                dest.set_extension(APP_EXT);
                println!("Converting: {path:?} -> {dest:?}\n");
                HskFile::convert(path, dest.as_path())?;
            }
        }
    }
    Ok(())
}
