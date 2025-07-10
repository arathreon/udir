use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Clone)]
struct FileToCopy {
    source: PathBuf,
    target: PathBuf,
}

#[derive(Debug, PartialEq, Clone)]
struct DirectoryToCreate {
    path: PathBuf,
}

#[derive(Debug, PartialEq)]
struct FilesAndDirectories {
    files: Vec<FileToCopy>,
    directories: Vec<DirectoryToCreate>,
}

///
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

/// Create directories from the provided vector of DirectoryToCreate structs
fn create_directories(list_of_directories: &Vec<DirectoryToCreate>) -> Vec<DirectoryToCreate> {
    let len_directories = list_of_directories.len();

    if (len_directories == 0) {
        return Vec::new();
    }

    let mut failed_directories: Vec<DirectoryToCreate> = vec![];

    for (i, directory) in list_of_directories.iter().enumerate() {
        print!(
            "\rCreating directories: {:.2}% ({}/{})",
            i as f64 / len_directories as f64 * 100.,
            i,
            len_directories
        );
        // Make sure it flushes immediately
        std::io::Write::flush(&mut io::stdout()).unwrap();
        match fs::create_dir(&directory.path) {
            Ok(_) => println!("\rDirectory created: {}", directory.path.display()),
            Err(_) => failed_directories.push(directory.clone()),
        }
    }

    println!(
        "\rCreating directories: 100.00% ({}/{})",
        len_directories, len_directories,
    );
    failed_directories
}

// /// Copy files from the provided vector of FileToCopy structs
// fn copy_files(list_of_files: &Vec<FileToCopy>) -> Vec<FileToCopy> {
//     let len_files = list_of_files.len();
//
//     if (len_files == 0) {
//         return Vec::new();
//     }
//
//     let mut failed_files = Vec::new();
//
//     for (i, file) in list_of_files.iter().enumerate() {
//         print!(
//             "\rCopying files: {:.2}% ({}/{})",
//             i as f64 / len_files as f64 * 100.,
//             i,
//             len_files
//         );
//         // Make sure it flushes immediately
//         std::io::Write::flush(&mut io::stdout()).unwrap();
//         match fs::copy(&file.source, &file.target) {
//             Ok(_) => println!("\rFile copied: {}", file.source.display()),
//             Err(_) => failed_files.push(file.clone()),
//         }
//     }
//     println!("\rCopying files: 100.00% ({}/{})", len_files, len_files);
//
//     failed_files
// }

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
    use std::thread::sleep;
    use std::time::Duration;

    use super::*;

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

        let mut results = get_files_and_directories(&source_dir_path, &target_dir_path).unwrap();

        results.files.sort_by_key(|val| val.source.clone());

        assert_eq!(
            results,
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

    #[test]
    fn test_create_directories() {
        // Set up files
        let current_path = env::current_dir().unwrap();
        let test_dir_path = current_path.join("test_dir");
        let existing_dir_path = test_dir_path.join("existing_dir");

        // Delete all test directories and files
        match fs::remove_dir_all(&test_dir_path) {
            Ok(_) => {}
            Err(_) => println!("[INFO] Test dir couldn't be removed"),
        };

        // Create test directories
        fs::create_dir(&test_dir_path).unwrap();
        fs::create_dir(&existing_dir_path).unwrap();

        // Test setup
        let test_input = vec![
            DirectoryToCreate {
                path: test_dir_path.join("test_path_1"),
            },
            DirectoryToCreate {
                path: test_dir_path.join("test_path_2"),
            },
            DirectoryToCreate {
                path: test_dir_path.join("test_path_2/inner_test_path_2_1"),
            },
            DirectoryToCreate {
                path: test_dir_path.join("test_path_2/inner_test_path_2_2"),
            },
            DirectoryToCreate {
                path: existing_dir_path.clone(), // Existing path
            },
            DirectoryToCreate {
                path: test_dir_path.join("test_path_3/inner_test_path_3_1"), // Path without existing parent folder
            },
        ];

        let expected_existing_directories = vec![
            test_dir_path.join("test_path_1"),
            test_dir_path.join("test_path_2"),
            test_dir_path.join("test_path_2/inner_test_path_2_1"),
            test_dir_path.join("test_path_2/inner_test_path_2_2"),
            existing_dir_path.clone(),
        ];

        let expected_failed_directories = vec![
            DirectoryToCreate {
                path: existing_dir_path.clone(), // Existing path
            },
            DirectoryToCreate {
                path: test_dir_path.join("test_path_3/inner_test_path_3_1"), // Path without existing parent folder
            },
        ];

        // Run the tested function
        let result = create_directories(&test_input);

        // Check that all directories that are expected to be created exist
        assert_eq!(result, expected_failed_directories);
        for i in expected_existing_directories.iter() {
            assert!(fs::exists(i).is_ok())
        }
        // Check that the one which is not expected to exist doesn't exist
        assert!(
            !fs::exists(test_dir_path.join("test_path_3/inner_test_path_3_1"))
                .expect("Directory doesn't exist")
        );

        // Delete all test directories and files
        fs::remove_dir_all(test_dir_path).unwrap();
    }

    // #[test]
    // fn test_copy_files() {
    //     let current_path = env::current_dir().unwrap();
    //     let test_dir_path = current_path.join("test_dir");
    //     let source_dir_path = test_dir_path.join("source_dir");
    //     let source_subdir_1_path = source_dir_path.join("subdir_subdir_1");
    //     let source_subdir_2_path = source_dir_path.join("subdir_subdir_2");
    //     let target_dir_path = test_dir_path.join("target_dir");
    // }
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

    let len_files = files.len();

    let failed_directories = create_directories(&directories);

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
    println!("\rCopying files: 100.00% ({}/{})", len_files, len_files);

    if failed_directories.len() > 0 {
        println!("Failed to create directories:");
        for directory in failed_directories {
            println!("    {}", directory.path.display());
        }
    }

    for directory in directories {
        println!("Directory created: {}", directory.path.display());
    }

    println!("Failed to create files:");
    for file in files {
        println!("File copied: {}", file.source.display());
    }
}
