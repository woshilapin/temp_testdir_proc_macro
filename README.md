If you compile the project with

```
cargo test -- --nocapture
```

Then you will see the path being injected and printed on stdout.

TODO: Improve the error message when the test function has no parameter `path:
&Path`. This is the current message
```
error[E0061]: this function takes 0 parameters but 1 parameter was supplied
--> tests/tempdir.rs:22:28
   |
22 | #[test_with_tempdir(ignore)]
   |                            ^ expected 0 parameters
23 | fn this_test_is_ignored() {
   | ------------------------- defined here

error: aborting due to previous error

For more information about this error, try `rustc --explain E0061`.
error: Could not compile `test_with_tempdir`.

To learn more, run the command again with --verbose.

		}
```
