use with_tempdir_procmacro::with_tempdir;

#[with_tempdir]
fn no_arg_fn(s: &String) {
    unimplemented!()
}

fn main() {
    assert!(true);
}
