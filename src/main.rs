mod file_handling;

use clap::Parser;
use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[
    command(
        version,
        about = "A simple utility for recursively updating a target directory from a source directory based on its 'modified' timestamp.",
        long_about = None,
    )
]
struct Cli {
    #[arg(help = "Source directory to copy from")]
    source: PathBuf,

    #[arg(help = "Target directory to copy to")]
    target: PathBuf,

    #[arg(long, help = "Add directories to skip (absolute or relative to SOURCE)", num_args = 1..)]
    skip_dir: Option<Vec<PathBuf>>,
}

fn main_inner(source: PathBuf, target: PathBuf, directories_to_skip: HashSet<PathBuf>) {
    let results = file_handling::get_files_and_directories(&source, &target, &directories_to_skip)
        .expect("Files and directories could not be generated!");
    let files = results.files;
    let directories = results.directories;

    let failed_directories = file_handling::create_directories(&directories);
    let failed_files = file_handling::copy_files(&files);

    if !failed_files.is_empty() {
        println!("Failed to create directories:");
        for directory in failed_directories {
            println!("    {}", directory.path.display());
        }
    }

    if !failed_files.is_empty() {
        println!("Failed to copy files:");
        for file in failed_files {
            println!("    {}", file.source.display());
        }
    }
}

/// Extracts the directories to skip from the provided `skip_dir` argument and returns them as a `HashSet<PathBuf>`.
/// Only directories that exist are added to the returned HashSet.
fn extract_skipped_directories(
    source: &Path,
    skip_dirs: &Option<Vec<PathBuf>>,
) -> HashSet<PathBuf> {
    let mut skipped_directories = HashSet::new();
    if let Some(skip_dirs) = skip_dirs {
        for skip_dir in skip_dirs {
            // This makes sure that the path is always either added to the source path or that it's
            // absolute
            let skip_dir_path = source.join(skip_dir);
            if skip_dir_path.is_dir() {
                skipped_directories.insert(skip_dir_path);
            }
        }
    }
    skipped_directories
}

fn main() {
    let cli = Cli::parse();

    let cwd = env::current_dir().unwrap();

    // Join adds a relative path to the current working directory or replaces with an absolute path.
    let source = cwd.join(cli.source);
    let target = cwd.join(cli.target);

    // We cannot do anything if the source or target directories don't exist, so we check that early
    // and exit if they are not directories.
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

    let directories_to_skip = extract_skipped_directories(&source, &cli.skip_dir);
    if !directories_to_skip.is_empty() {
        println!("Directories to skip:");
        for directory in &directories_to_skip {
            println!("    {}", directory.display());
        }
    } else {
        println!("No directories to skip");
    }

    main_inner(source, target, directories_to_skip);
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::thread::sleep;
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_main_inner() {
        // Set up files
        let current_path = env::current_dir().unwrap();
        let test_dir_path = current_path.join("test_dir_main_inner");
        let source_dir_path = test_dir_path.join("source_dir");
        let source_subdir_1_path = source_dir_path.join("subdir_1");
        let source_subdir_2_path = source_dir_path.join("subdir_2");
        let source_subdir_4_path = source_dir_path.join("subdir_4");
        let target_dir_path = test_dir_path.join("target_dir");
        let target_subdir_1_path = target_dir_path.join("subdir_1");
        let target_subdir_2_path = target_dir_path.join("subdir_2");
        let target_subdir_3_path = target_dir_path.join("subdir_3");

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
        fs::create_dir(&source_subdir_4_path).unwrap();
        fs::create_dir(&target_dir_path).unwrap();
        fs::create_dir(&target_subdir_1_path).unwrap();
        fs::create_dir(&target_subdir_3_path).unwrap();

        // Write files where the target should be overwritten
        let target_file_1 = target_dir_path.join("test_1.txt");
        let source_file_1 = source_dir_path.join("test_1.txt");
        let source_file_1_content = b"This is new text in file 1";
        fs::write(&target_file_1, b"This is old text in file 1").unwrap();
        sleep(Duration::from_nanos(1)); // waiting so the source file is newer
        fs::write(&source_file_1, source_file_1_content).unwrap();

        // Write files that should stay the same
        let target_file_2 = target_dir_path.join("test_2.txt");
        let source_file_2 = source_dir_path.join("test_2.txt");
        let source_file_2_content = b"This is unchanged text in file 2";
        fs::write(&target_file_2, source_file_2_content).unwrap();
        fs::copy(&target_file_2, &source_file_2).unwrap();
        assert_eq!(
            fs::metadata(&source_file_2).unwrap().modified().unwrap(),
            fs::metadata(&target_file_2).unwrap().modified().unwrap(),
        );

        // Write files that should stay the same in subdirectory 1
        let target_file_3 = target_subdir_1_path.join("test_3.txt");
        let source_file_3 = source_subdir_1_path.join("test_3.txt");
        let source_file_3_content = b"This is unchanged text in file 3";
        fs::write(&target_file_3, source_file_3_content).unwrap();
        fs::copy(&target_file_3, &source_file_3).unwrap();
        assert_eq!(
            fs::metadata(&source_file_3).unwrap().modified().unwrap(),
            fs::metadata(&target_file_3).unwrap().modified().unwrap(),
        );

        // Write files that should be changed in subdirectory 1
        let target_file_4 = target_subdir_1_path.join("test_4.txt");
        let source_file_4 = source_subdir_1_path.join("test_4.txt");
        let source_file_4_content = b"This is new text in file 4";
        fs::write(&target_file_4, b"This is old text in file 4").unwrap();
        sleep(Duration::from_nanos(1)); // waiting so the source file is newer
        fs::write(&source_file_4, source_file_4_content).unwrap();

        // Write a file that should be created in subdirectory 1
        let target_file_5 = target_subdir_1_path.join("test_5.txt");
        let source_file_5 = source_subdir_1_path.join("test_5.txt");
        let source_file_5_content = b"This is new text in file 5";
        fs::write(&source_file_5, source_file_5_content).unwrap();

        // Write a file that should be created in subdirectory 2
        let target_file_6 = target_subdir_2_path.join("test_6.txt");
        let source_file_6 = source_subdir_2_path.join("test_6.txt");
        let source_file_6_content = b"This is new text in file 6";
        fs::write(&source_file_6, source_file_6_content).unwrap();

        // Write a file that should stay in target subdirectory 3
        let target_file_7 = target_subdir_3_path.join("test_7.txt");
        let target_file_7_content = b"This is a relict that should not be touched file 7";
        fs::write(&target_file_7, target_file_7_content).unwrap();

        // Run the tested function
        main_inner(
            source_dir_path.clone(),
            target_dir_path.clone(),
            HashSet::new(),
        );

        // Verify directory structure
        assert!(
            target_subdir_1_path.exists(),
            "Existing directory should remain"
        );
        assert!(
            target_subdir_2_path.exists(),
            "New directory should be created"
        );
        assert!(
            target_subdir_3_path.exists(),
            "Existing directory should remain"
        );
        assert!(
            target_dir_path.join("subdir_4").exists(),
            "New directory should be created"
        );

        // Verify source files remain unchanged
        assert_eq!(
            fs::read(&source_file_1).unwrap(),
            source_file_1_content,
            "Source file 1 should remain unchanged"
        );
        assert_eq!(
            fs::read(&source_file_2).unwrap(),
            source_file_2_content,
            "Source file 2 should remain unchanged"
        );
        assert_eq!(
            fs::read(&source_file_3).unwrap(),
            source_file_3_content,
            "Source file 3 should remain unchanged"
        );
        assert_eq!(
            fs::read(&source_file_4).unwrap(),
            source_file_4_content,
            "Source file 4 should remain unchanged"
        );
        assert_eq!(
            fs::read(&source_file_5).unwrap(),
            source_file_5_content,
            "Source file 5 should remain unchanged"
        );
        assert_eq!(
            fs::read(&source_file_6).unwrap(),
            source_file_6_content,
            "Source file 6 should remain unchanged"
        );

        // Check that files expected to exist actually exist and have the correct content.
        assert_eq!(
            fs::read(&target_file_1).unwrap(),
            source_file_1_content,
            "Target file 1 should be updated"
        );
        assert_eq!(
            fs::read(&target_file_2).unwrap(),
            source_file_2_content,
            "Target file 2 should remain unchanged"
        );
        assert_eq!(
            fs::read(&target_file_3).unwrap(),
            source_file_3_content,
            "Target file 3 should remain unchanged"
        );
        assert_eq!(
            fs::read(&target_file_4).unwrap(),
            source_file_4_content,
            "Target file 4 should be updated"
        );
        assert_eq!(
            fs::read(&target_file_5).unwrap(),
            source_file_5_content,
            "Target file 5 should be created"
        );
        assert_eq!(
            fs::read(&target_file_6).unwrap(),
            source_file_6_content,
            "Target file 6 should be created"
        );
        assert_eq!(
            fs::read(&target_file_7).unwrap(),
            target_file_7_content,
            "Target file 7 should remain unchanged"
        );

        // Delete all test directories and files
        fs::remove_dir_all(test_dir_path).unwrap();
    }

    #[test]
    fn test_extract_skipped_directories_receives_none() {
        let source = PathBuf::from("source");
        assert_eq!(extract_skipped_directories(&source, &None), HashSet::new());
    }

    #[test]
    fn test_extract_skipped_directories_receives_some() {
        // Test setup
        let current_path = env::current_dir().unwrap();
        let test_dir_path = current_path.join("test_dir_extract_skipped_directories");

        let subdir_1_path = PathBuf::from("subdir_1");
        let subdir_5_path = PathBuf::from("subdir_5");

        let source_subdir_1_path = test_dir_path.join(subdir_1_path.clone());
        let source_subdir_2_path = test_dir_path.join("subdir_2");
        let source_subdir_3_path = test_dir_path.join("subdir_3");
        let source_subdir_4_path = test_dir_path.join("subdir_4");

        fs::create_dir(&test_dir_path).unwrap();
        fs::create_dir(&source_subdir_1_path).unwrap();
        fs::create_dir(&source_subdir_3_path).unwrap();
        fs::create_dir(&source_subdir_4_path).unwrap();

        // Define skipped directories, all possible states
        let skip_dirs = vec![
            subdir_1_path.clone(),        // existing relative path
            source_subdir_2_path.clone(), // non-existent absolute path
            source_subdir_3_path.clone(), // existing absolute path
            subdir_5_path.clone(),        // non-existent relative path
        ];

        // Expect only existing directories and only their absolute paths
        let result = HashSet::from([
            source_subdir_1_path, // from an existing relative path
            source_subdir_3_path, // from an existing absolute path
        ]);

        assert_eq!(
            extract_skipped_directories(&test_dir_path, &Some(skip_dirs)),
            result
        );

        fs::remove_dir_all(test_dir_path).unwrap();
    }

    #[test]
    fn paths_are_the_same() {
        let source = PathBuf::from("source");
        let path1 = source.join("dir1");
        let path2 = source.join("dir1/");
        assert_eq!(path1, path2);
    }

    #[test]
    fn path_hashes_are_the_same() {
        let source = PathBuf::from("source");
        let path1 = source.join("dir1");
        let path2 = source.join("dir1/");
        let hash_set = HashSet::from([path1, path2]);
        assert_eq!(hash_set.len(), 1);
    }
}
