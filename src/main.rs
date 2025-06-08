use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
struct FileToCopy {
    source: PathBuf,
    target: PathBuf,
}

#[derive(Debug, PartialEq)]
struct DirectoryToCreate {
    path: PathBuf,
}

#[derive(Debug, PartialEq)]
struct FilesAndDirectories {
    files: Vec<FileToCopy>,
    directories: Vec<DirectoryToCreate>,
}

fn get_files_and_directories(
    source: &PathBuf,
    target: &PathBuf,
) -> io::Result<FilesAndDirectories> {
    let mut files = Vec::new();
    let mut directories = Vec::new();

    if source.is_dir() {
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let source_path = entry.path();
            if source_path.is_dir() {
                // If the source_path is a subdirectory, check, whether it exists. If not, add it
                // to be created. Call the function on the subdirectory.
                let dir_name = source_path.file_name().unwrap();
                let new_target = Path::new(target).join(Path::new(dir_name));
                let dir_exists = fs::exists(&new_target)?;
                if !dir_exists {
                    directories.push(DirectoryToCreate {
                        path: new_target.clone(),
                    });
                }
                let mut result = get_files_and_directories(&source_path, &new_target)?;
                files.append(&mut result.files);
                directories.append(&mut result.directories);
            } else {
                // Source path is a file
                let file_name = source_path.file_name().unwrap();
                let target_path = Path::new(target).join(Path::new(file_name));
                let file_exists = fs::exists(&target_path)?;

                if file_exists {
                    // If the target directory contains a file with the same name as the source path,
                    // check last modified timestamps. If the source file was modified later, re-write
                    // the target file.
                    let source_metadata = fs::metadata(&source_path)?;
                    let target_metadata = fs::metadata(&target_path)?;

                    let source_last_modified = source_metadata.modified()?;
                    let target_last_modified = target_metadata.modified()?;

                    if target_last_modified < source_last_modified {
                        files.push(FileToCopy {
                            source: source_path,
                            target: target_path,
                        });
                    }
                } else {
                    // If the target path doesn't exist, copy the source path.
                    files.push(FileToCopy {
                        source: source_path,
                        target: target_path,
                    });
                }
            }
        }
    }
    Ok(FilesAndDirectories { files, directories })
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;
    #[test]
    fn test_get_files_and_directories() {
        // Set up files
        let current_path = env::current_dir().unwrap();
        let test_dir_path = current_path.join("test_dir");
        let source_dir_path = test_dir_path.join("source_dir");
        let source_subdir_1_path = source_dir_path.join("subdir_subdir_1");
        let source_subdir_2_path = source_dir_path.join("subdir_subdir_2");
        let target_dir_path = test_dir_path.join("target_dir");
        let target_subdir_1_path = target_dir_path.join("subdir_subdir_1");
        let target_subdir_2_path = target_dir_path.join("subdir_subdir_2");
        let target_subdir_3_path = target_dir_path.join("subdir_subdir_3");

        // Delete all test directories and files
        match fs::remove_dir_all(&test_dir_path) {
            Ok(_) => {}
            Err(_) => println!("[INFO] Test dir couldn't be removed"),
        };

        // Create test directories
        fs::create_dir(&test_dir_path).unwrap();
        fs::create_dir(&source_dir_path).unwrap();
        fs::create_dir(&source_subdir_1_path).unwrap();
        fs::create_dir(&source_subdir_2_path).unwrap();
        fs::create_dir(&target_dir_path).unwrap();
        fs::create_dir(&target_subdir_1_path).unwrap();
        fs::create_dir(&target_subdir_3_path).unwrap();

        // Write files where target should be overwritten
        let target_file_1 = target_dir_path.join("test_1.txt");
        let source_file_1 = source_dir_path.join("test_1.txt");
        let source_file_1_content = b"This is some newer text";
        fs::write(&target_file_1, b"This is some text").unwrap();
        sleep(Duration::from_nanos(1)); // waiting so the source file is newer
        fs::write(&source_file_1, &source_file_1_content).unwrap();

        // Write files that should stay the same
        let target_file_2 = target_dir_path.join("test_2.txt");
        let source_file_2 = source_dir_path.join("test_2.txt");
        let source_file_2_content = b"This is unchanged text";
        fs::write(&target_file_2, source_file_2_content).unwrap();
        fs::copy(&target_file_2, &source_file_2).unwrap();
        assert_eq!(
            fs::metadata(&source_file_2).unwrap().modified().unwrap(),
            fs::metadata(&target_file_2).unwrap().modified().unwrap(),
        );

        // Write files that should stay the same in subdirectory 1
        let target_file_3 = target_subdir_1_path.join("test_3.txt");
        let source_file_3 = source_subdir_1_path.join("test_3.txt");
        let source_file_3_content = b"This is unchanged text too";
        fs::write(&target_file_3, &source_file_3_content).unwrap();
        fs::copy(&target_file_3, &source_file_3).unwrap();
        assert_eq!(
            fs::metadata(&source_file_3).unwrap().modified().unwrap(),
            fs::metadata(&target_file_3).unwrap().modified().unwrap(),
        );

        // Write files that should be changed in subdirectory 1
        let target_file_4 = target_subdir_1_path.join("test_4.txt");
        let source_file_4 = source_subdir_1_path.join("test_4.txt");
        let source_file_4_content = b"4 This is some changed text in subdirectory 1";
        fs::write(&target_file_4, b"4 This is some text in subdirectory 1").unwrap();
        sleep(Duration::from_nanos(1)); // waiting so the source file is newer
        fs::write(&source_file_4, &source_file_4_content).unwrap();

        // Write a file that should be created in subdirectory 1
        let target_file_5 = target_subdir_1_path.join("test_5.txt");
        let source_file_5 = source_subdir_1_path.join("test_5.txt");
        let source_file_5_content = b"5 This is some new text in subdirectory 1";
        fs::write(&source_file_5, &source_file_5_content).unwrap();

        // Write a file that should be created in subdirectory 2
        let target_file_6 = target_subdir_2_path.join("test_6.txt");
        let source_file_6 = source_subdir_2_path.join("test_6.txt");
        let source_file_6_content = b"6 This is some new text in subdirectory 1";
        fs::write(&source_file_6, &source_file_6_content).unwrap();

        // Write a file that should stay in target subdirectory 3
        let target_file_7 = target_subdir_3_path.join("test_7.txt");
        let target_file_7_content = b"7 This is a relict that should not be touched";
        fs::write(&target_file_7, &target_file_7_content).unwrap();

        let results = get_files_and_directories(&source_dir_path, &target_dir_path);

        assert_eq!(
            results.unwrap(),
            FilesAndDirectories {
                files: vec![
                    FileToCopy {
                        source: source_file_4,
                        target: target_file_4,
                    },
                    FileToCopy {
                        source: source_file_5,
                        target: target_file_5,
                    },
                    FileToCopy {
                        source: source_file_6,
                        target: target_file_6,
                    },
                    FileToCopy {
                        source: source_file_1,
                        target: target_file_1,
                    },
                ],
                directories: vec![DirectoryToCreate {
                    path: target_subdir_2_path,
                }]
            }
        );

        // Delete all test directories and files
        fs::remove_dir_all(test_dir_path).unwrap();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let source;
    let target;
    if args.len() > 2 {
        source = PathBuf::new().join(&args[1]);
        target = PathBuf::new().join(&args[2]);
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

    let directories;
    let files;
    let results = get_files_and_directories(&source, &target)
        .expect("Files and directories could not be generated!");
    files = results.files;
    directories = results.directories;

    let len_directories = directories.len();
    let len_files = files.len();

    for (i, directory) in directories.iter().enumerate() {
        print!(
            "\rCreating directories: {:.2}% ({}/{})",
            i as f64 / len_directories as f64 * 100.,
            i,
            len_directories
        );
        // Make sure it flushes immediately
        std::io::Write::flush(&mut io::stdout()).unwrap();
        fs::create_dir(&directory.path).unwrap();
    }

    println!(
        "\rCreating directories: 100.00% ({}/{})",
        len_directories, len_directories,
    );

    for (i, file) in files.iter().enumerate() {
        print!(
            "\rCopying files: {:.2}% ({}/{})",
            i as f64 / len_files as f64 * 100.,
            i,
            len_files
        );
        // Make sure it flushes immediately
        std::io::Write::flush(&mut io::stdout()).unwrap();
        fs::copy(&file.source, &file.target).unwrap();
    }
    println!("\rCopying files: 100.00% ({}/{})", len_files, len_files,);

    for directory in directories {
        println!("Directory created: {}", directory.path.display());
    }

    for file in files {
        println!("File copied: {}", file.source.display());
    }
}
