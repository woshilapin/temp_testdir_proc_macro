use pretty_assertions::assert_eq;
use regex::Regex;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use test_with_tempdir::test_with_tempdir;

#[test_with_tempdir]
#[test]
fn path_exists(path: &Path) {
    assert_eq!(path.exists(), true);
}

#[test_with_tempdir]
#[test]
fn write_and_read(path: &Path) {
    let file_path = path.join("some_file.txt");
    let mut file = File::create(&file_path).expect("Failed to create the file");
    file.write_fmt(format_args!("some content"))
        .expect("Failed to write in the file");
    assert_eq!(file_path.exists(), true);
    let mut file = File::open(&file_path).expect("Failed to create the file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Failed to read the file");
    assert_eq!(content, String::from("some content"));
}

#[test_with_tempdir]
#[test]
#[ignore]
fn is_ignored(_path: &Path) {
    assert!(false);
}

#[test_with_tempdir(path = "./tests")]
#[test]
fn with_path(path: &Path) {
    let regex = Regex::new("tests").unwrap();
    assert!(regex.is_match(path.to_str().unwrap()));
}

#[test_with_tempdir(path = b"./tests")]
#[test]
fn with_bytes_path(path: &Path) {
    let regex = Regex::new("tests").unwrap();
    assert!(regex.is_match(path.to_str().unwrap()));
}

#[test_with_tempdir(path = "/tmp/foo/bar")]
#[test]
#[should_panic]
fn with_folder_not_existing_inner(_path: &Path) {
    assert!(true);
}

#[test_with_tempdir]
#[test]
#[should_panic]
fn should_panic(_path: &Path) {
    let option: Option<u8> = None;
    option.unwrap();
}
