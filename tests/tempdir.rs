use test_with_tempdir::test_with_tempdir;
use std::path::Path;

#[test_with_tempdir]
fn some_test(path: &Path) {
    dbg!(path);
    assert_eq!(true, true);
}
