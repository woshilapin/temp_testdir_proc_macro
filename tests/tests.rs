use pretty_assertions::assert_eq;
use regex::Regex;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use with_tempdir_procmacro::with_tempdir;

#[test]
fn compile_error() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/01-function-with-no-arg.rs");
    t.compile_fail("tests/02-function-with-wrong-type-arg.rs");
    t.compile_fail("tests/03-function-with-wrong-ref-type-arg.rs");
    t.pass("tests/04-has-working-path.rs");
}

#[with_tempdir]
#[test]
fn write_and_read(path: &Path) {
    assert_eq!(path.exists(), true);
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

#[with_tempdir]
#[test]
#[ignore]
fn is_ignored(_path: &Path) {
    assert!(false);
}

#[with_tempdir(path = "./tests")]
#[test]
fn with_path(path: &Path) {
    let regex = Regex::new("tests").unwrap();
    assert!(regex.is_match(path.to_str().unwrap()));
}

#[with_tempdir(path = b"./tests")]
#[test]
fn with_bytes_path(path: &Path) {
    let regex = Regex::new("tests").unwrap();
    assert!(regex.is_match(path.to_str().unwrap()));
}

#[with_tempdir(path = "/tmp/foo/bar")]
#[test]
#[should_panic]
fn with_folder_not_existing_inner(_path: &Path) {
    assert!(true);
}

#[with_tempdir]
#[test]
#[should_panic]
fn should_panic(_path: &Path) {
    let option: Option<u8> = None;
    option.unwrap();
}
