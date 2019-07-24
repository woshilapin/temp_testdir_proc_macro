use pretty_assertions::assert_eq;
use regex::Regex;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use test_with_tempdir::test_with_tempdir;

#[test_with_tempdir]
fn path_exists(path: &Path) {
    assert_eq!(path.exists(), true);
}

#[test_with_tempdir]
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

#[test_with_tempdir(ignore)]
fn this_test_is_ignored(_path: &Path) {
    assert!(false);
}

#[test_with_tempdir(path = "/tmp/foo")]
fn this_test_is_ignored_with_path(path: &Path) {
    let regex = Regex::new("^/tmp/foo").unwrap();
    assert!(regex.is_match(path.to_str().unwrap()));
}

#[test_with_tempdir(path = b"/tmp/foo")]
fn this_test_is_ignored_with_bytes_path(path: &Path) {
    let regex = Regex::new("^/tmp/bar").unwrap();
    assert!(regex.is_match(path.to_str().unwrap()));
}
