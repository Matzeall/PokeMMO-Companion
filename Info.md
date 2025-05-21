# RUST Concepts

## modules & sub-modules

- mod foo; tells Rust “load src/foo.rs as module foo.”

- mod bar { … } defines a module inline inside the current file—you can nest as deeply as you like without extra files.

- mod backend; + src/backend/mod.rs + backend/storage.rs → exposes backend::storage.

- use ... as alias; lets you shorten long paths in your code.
