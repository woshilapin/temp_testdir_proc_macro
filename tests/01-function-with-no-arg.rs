use with_tempdir_procmacro::with_tempdir;

#[with_tempdir]
fn no_arg_fn() {
    unimplemented!()
}

fn main() {
    assert!(true);
}
