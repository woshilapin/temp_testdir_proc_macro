use std::path::Path;
use test_with_tempdir::test_with_tempdir;

#[test_with_tempdir]
fn some_test(path: &Path) {
    dbg!(path);
    assert_eq!(true, true);
}
#[test_with_tempdir]
fn some_test_other(path: &Path) {
    dbg!(path);
    assert_eq!(true, true);
}
