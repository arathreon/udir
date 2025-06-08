use std::env;
use std::fs;
use std::io;
use std::path::Path;

fn update_files_in_directory(source: &Path, target: &Path) -> io::Result<Vec<String>> {
    let mut copied_paths = vec![];

    if source.is_dir() && target.is_dir() {
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let source_path = entry.path();
            // If the current source path is a directory, check whether such subdirectory exists
            // in the target path. If not, create it. Call the function for the subdirectories.
            if source_path.is_dir() {
                let dir_name = source_path.file_name().unwrap();
                let new_target = Path::new(target).join(Path::new(dir_name));
                let dir_exists = fs::exists(&new_target)?;
                if !dir_exists {
                    fs::create_dir(&new_target)?;
                }
                let mut copied_paths_in_dir = update_files_in_directory(&source_path, &new_target)?;
                copied_paths.append(&mut copied_paths_in_dir);
            } else {
                // Source path is a file
                let file_name = source_path.file_name().unwrap();
                let target_path = Path::new(target).join(Path::new(file_name));
                let file_exists = fs::exists(&target_path)?;

                // If the target directory contains a file with the same name as the source path,
                // check last modified timestamps. If the source file was modified later, re-write
                // the target file.
                if file_exists {
                    let source_metadata = fs::metadata(&source_path)?;
                    let target_metadata = fs::metadata(&target_path)?;

                    let source_last_modified = source_metadata.modified()?;
                    let target_last_modified = target_metadata.modified()?;

                    if target_last_modified < source_last_modified {
                        fs::copy(source_path, &target_path).expect("File could not be copied");
                        copied_paths.push(target_path.into_os_string().into_string().unwrap());
                    }
                // If the target path doesn't exist, copy the source path.
                } else {
                    fs::copy(source_path, &target_path).expect("File could not be copied");
                    copied_paths.push(target_path.into_os_string().into_string().unwrap());
                }
            }
        }
    }
    Ok(copied_paths)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let source;
    let target;
    if args.len() > 2 {
        source = Path::new(&args[1]);
        target = Path::new(&args[2]);
    } else {
        println!("Insufficient number of input arguments.");
        return;
    }

    if !source.is_dir() {
        println!("Source {} is not a directory", &source.display());
        return;
    }

    if !target.is_dir() {
        println!("Target {} is not a directory", &target.display());
        return;
    }

    println!("Source dir: {}", &source.display());
    println!("Target dir: {}", &target.display());

    let copied_paths = update_files_in_directory(&source, &target)
        .expect("There was a problem with traversing the directory tree.");

    for copied_path in copied_paths {
        println!("Copied {}", copied_path);
    }
}
