# RUST Concepts

## modules & sub-modules

- `mod foo;` tells Rust “load src/foo.rs as module foo.”

- `mod bar { … }` defines a module inline inside the current file—you can nest as deeply as you like without extra files.

- `mod backend;` + src/backend/mod.rs + backend/storage.rs → exposes backend::storage.

- `use ... as alias`; lets you shorten long paths in your code.

## Ownership 
- Move (caller loses ownership)

  - Function signature: `fn foo(x: T)` where T is an owned (non-Copy) type (e.g. String, custom struct, Vec<T>, etc.).

  - Result: Calling `foo(my_value)` transfers ownership into foo; my_value is no longer valid afterward.

- Borrow (caller keeps ownership)

  - Function signature: fn `foo(x: &T)` (or &mut T if you need mutation).

  - Result: Calling `foo(&my_value)` or `foo(&mut my_value)` temporarily lends it; the caller still owns and can use my_value once foo returns.

> In the Rust code, the moment you use `&`, you’re in the world of safe borrowing, not raw address manipulation. 
Only inside unsafe (with `*const T` or `*mut T`) do you get to mimic C’s `&`/`*` pointer arithmetic, and even then it’s explicitly marked as unsafe.

## The usual Rust patterns are:

- Short-lived borrows. Call your &mut method, let its borrow end (i.e. don’t store a &mut in a variable unless you really need it), then call the next one.

- Split your data. Break your big struct into smaller sub-objects so that each method only borrows one sub-object, and those borrows don’t overlap.

- Interior mutability. When you must hold on to multiple mutable borrows in the same scope, wrap the fields in RefCell/Mutex so the compiler sees only shared borrows.


## Rc/Arc vs. RefCell

### Rc<T> / Arc<T>

- These are **reference-counted** pointers for shared ownership of an heap-allocated object.

- **Rc & Arc only provide immutable access!!** to mutate them T needs to be wrapped in a RefCell (or Mutex etc for Arc)
  - Box<T> is the not reference-counted version which does allow for mutable access (comparable to unique_ptr in C++)

- Rc<T> is single-threaded; Arc<T> is thread-safe (atomic).

- They let you clone handles to the same data so that multiple owners can keep it alive.

### RefCell<T>

- Gives you interior mutability—the ability to mutate data behind a shared (&) reference, checked at **runtime**.

A very common pattern in Rust GUI apps or async code is to combine them(because Rc is only immutable):
``` 
  use std::rc::Rc;             // or std::sync::Arc for multi-threaded
  use std::cell::RefCell;

  type SharedState = Rc<RefCell<MyState>>;

  let state: SharedState = Rc::new(RefCell::new(MyState::new()));
  // clone `state` and move into closures, callbacks, threads, etc.
  // inside each closure you can do:
  let mut borrow = state.borrow_mut();
  // … mutate the fields …
```

### Why both?

- Rc/Arc solves “how do I share ownership of the same data across callbacks/threads?”

- RefCell solves “how do I get mutable access to a field when all I have is a shared handle (&, &self)?”

>They’re orthogonal: one handles ownership and lifetime, the other handles mutation rules. 
In single-threaded GUI code you’ll often see Rc<RefCell<…>>; 
in multi-threaded contexts it becomes Arc<Mutex<…>> or Arc<RwLock<…>> (because RefCell is not thread safe).
