![Travis (.org)](https://img.shields.io/travis/woshilapin/with_tempdir_procmacro)

Tempdir injection for tests
===========================
This small project is providing you a procedural macro to inject a temporary
directory in your test.

```rust
#[with_tempdir]
#[test]
fn my_test(path: &Path) {
  // do stuff in folder `path`
}
```
