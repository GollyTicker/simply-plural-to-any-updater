# clippy/rustc bug

## How to reproduce

* ensure rust toolchain via rustup installed
* run `cargo build` to check, that code indeed compiles
* run `cargo clippy --allow-dirty --fix` to produce the error below

## Error

```
warning: failed to automatically apply fixes suggested by rustc to crate `blabla`

after fixes were automatically applied the compiler reported errors within these files:

  * src/main.rs

This likely indicates a bug in either rustc or cargo itself,
and we would appreciate a bug report! You're likely to see
a number of compiler warnings after this message which cargo
attempted to fix but failed. If you could open an issue at
https://github.com/rust-lang/rust-clippy/issues
quoting the full output of this command we'd be very appreciative!
Note that you may be able to make some more progress in the near-term
fixing code with the `--broken-code` flag

The following errors were reported:
error: expected expression, found `$`
 --> src/main.rs:6:7
  |
6 |       $crate::__log!(
  |       ^ expected expression

error: aborting due to 1 previous error

Original diagnostics will follow.

warning: passing a unit value to a function
 --> src/main.rs:5:3
  |
5 |   Ok(log::info!("{n}")) // ERROR
  |   ^^^^^^^^^^^^^^^^^^^^^
  |
  = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#unit_arg
  = note: `#[warn(clippy::unit_arg)]` on by default
help: move the expression in front of the call and replace it with the unit literal `()`
  |
5 ~   {
6 +       $crate::__log!(
7 +           logger: $crate::__log_logger!(__log_global_logger),
8 +           target: $crate::__private_api::module_path!(),
9 +           $lvl,
10+           $($arg)+
11+       )
12+   };
13~   Ok(log::info!("{n}")) // ERROR
  |

warning: `blabla` (bin "blabla") generated 1 warning (run `cargo clippy --fix --bin "blabla"` to apply 1 suggestion)
warning: failed to automatically apply fixes suggested by rustc to crate `blabla`

after fixes were automatically applied the compiler reported errors within these files:

  * src/main.rs

This likely indicates a bug in either rustc or cargo itself,
and we would appreciate a bug report! You're likely to see
a number of compiler warnings after this message which cargo
attempted to fix but failed. If you could open an issue at
https://github.com/rust-lang/rust-clippy/issues
quoting the full output of this command we'd be very appreciative!
Note that you may be able to make some more progress in the near-term
fixing code with the `--broken-code` flag

The following errors were reported:
error: expected expression, found `$`
 --> src/main.rs:6:7
  |
6 |       $crate::__log!(
  |       ^ expected expression

error: aborting due to 1 previous error

Original diagnostics will follow.

warning: `blabla` (bin "blabla" test) generated 1 warning (1 duplicate)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.67s
```

