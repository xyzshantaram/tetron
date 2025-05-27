# tetron development notes

## Module and Type Registration

- **Module Paths:** Create modules at a specific Rune path (e.g., `tetron::log`)
  using `Module::with_crate_item("tetron", ["log"])`, or with arbitrary item
  paths `Module::with_item(["a", "b"])`.
- **Flat Registration:** Rune does _not_ support hierarchical/nested submodules
  in registration: Register each module (including submodules) individually into
  the `Context`.
- **Types:** Add Rust type with `module.ty::<MyType>()?`.

---

## Naming Types

- Rename a type for the Rune side using `#[rune(name = NewName)]`:
  ```rust
  #[derive(Any)]
  #[rune(name = World)]
  struct WorldRef { /* ... */ }
  ```

---

## Function Registration and Rust Interop

- **Registering Functions:** Use `#[rune::function]` on a regular Rust function;
  register via `module.function_meta(my_function)?`.
- **Instance Methods & Statics:** Plain instance methods, statics
  (`#[rune::function(path = Self::new)]`), mutators (`fn foo(&mut self, ...)`),
  and operator overloads are all supported.

---

## Operator Overloading ("Protocols")

- Use the correct protocol in method attribute for operator overloads:
  ```rust
  #[rune::function(instance, protocol = ADD)]
  fn add(&self, rhs: &Self) -> Self { /* ... */ }
  ```
  This enables things like `a + b` in Rune if `a` and `b` are the type for which
  Protocol::ADD is defined.

---

## Cross-Calling (Rust <-> Rune)

- **Shared functions:** Any function registered with `#[rune::function]` gets
  its name mangled - if you want to call it easily from Rust, use
  rune::function(keep) and then while doing fn_meta() pass &lt;fnname&gt;__meta.

---

## Exposing Errors and VmResult

- VmResult is for fatal VM errors
- all other Result types can be passed to Rune as `Value`s

---

## Protocols

### **Field Access Protocols**

#### **`Protocol::GET`**

- **Usage**: Accessing fields or properties
- **Representation**: `let $out = $value`
- **Example**: `obj.field` or accessing struct fields

#### **`Protocol::SET`**

- **Usage**: Setting fields or properties
- **Representation**: `$value = $input`
- **Example**: `obj.field = 42`

---

### **Indexing Protocols**

#### **`Protocol::INDEX_GET`**

- **Usage**: Getting values by index
- **Representation**: `let $out = $value[index]`
- **Example**: `array[0]` or `map["key"]`

#### **`Protocol::INDEX_SET`**

- **Usage**: Setting values by index
- **Representation**: `$value[index] = $input`
- **Example**: `array[0] = 42`

---

### **Comparison Protocols**

#### **`Protocol::PARTIAL_EQ`**

- **Usage**: Partial equality comparison
- **Method**: `eq`
- **Representation**: `if $value == b { }`
- **Example**: `a == b`

#### **`Protocol::EQ`**

- **Usage**: Total equality comparison
- **Representation**: `if $value == b { }`
- **Example**: Used for strict equality

#### **`Protocol::PARTIAL_CMP`**

- **Usage**: Partial ordering comparison
- **Method**: `partial_cmp`
- **Representation**: `if $value < b { }`
- **Example**: Foundation for `<`, `>`, `<=`, `>=`

#### **`Protocol::CMP`**

- **Usage**: Total ordering comparison
- **Method**: `cmp`
- **Representation**: `if $value < b { }`
- **Example**: Used for sorting algorithms

#### **`Protocol::GT`**, **`Protocol::GE`**, **`Protocol::LT`**, **`Protocol::LE`**

- **Usage**: Specific comparison operators
- **Methods**: `gt`, `ge`, `lt`, `le`
- **Representations**: `if $a > $b { }`, `if $a >= $b { }`, etc.

#### **`Protocol::MAX`**, **`Protocol::MIN`**

- **Usage**: Finding max/min values
- **Methods**: `max`, `min`
- **Representations**: `$a.max($b)`, `$a.min($b)`

---

### **Arithmetic Protocols**

#### **`Protocol::ADD`** / **`Protocol::ADD_ASSIGN`**

- **Usage**: Addition operations
- **Representation**: `let $out = $value + $b` / `$value += $b`
- **Example**: `a + b` or `a += b`

#### **`Protocol::SUB`** / **`Protocol::SUB_ASSIGN`**

- **Usage**: Subtraction operations
- **Representation**: `let $out = $value - $b` / `$value -= $b`
- **Example**: `a - b` or `a -= b`

#### **`Protocol::MUL`** / **`Protocol::MUL_ASSIGN`**

- **Usage**: Multiplication operations
- **Representation**: `let $out = $value * $b` / `$value *= $b`
- **Example**: `a * b` or `a *= b`
- **Demo**: See `/home/sid/rune/examples/examples/custom_mul.rs`

#### **`Protocol::DIV`** / **`Protocol::DIV_ASSIGN`**

- **Usage**: Division operations
- **Representation**: `let $out = $value / $b` / `$value /= $b`
- **Example**: `a / b` or `a /= b`

#### **`Protocol::REM`** / **`Protocol::REM_ASSIGN`**

- **Usage**: Remainder operations
- **Representation**: `let $out = $value % $b` / `$value %= $b`
- **Example**: `a % b` or `a %= b`

---

### **Bitwise Protocols**

#### **`Protocol::BIT_AND`** / **`Protocol::BIT_AND_ASSIGN`**

- **Usage**: Bitwise AND operations
- **Representation**: `let $out = $value & $b` / `$value &= $b`

#### **`Protocol::BIT_XOR`** / **`Protocol::BIT_XOR_ASSIGN`**

- **Usage**: Bitwise XOR operations
- **Representation**: `let $out = $value ^ $b` / `$value ^= $b`

#### **`Protocol::BIT_OR`** / **`Protocol::BIT_OR_ASSIGN`**

- **Usage**: Bitwise OR operations
- **Representation**: `let $out = $value | $b` / `$value |= $b`

#### **`Protocol::SHL`** / **`Protocol::SHL_ASSIGN`**

- **Usage**: Left shift operations
- **Representation**: `let $out = $value << $b` / `$value <<= $b`

#### **`Protocol::SHR`** / **`Protocol::SHR_ASSIGN`**

- **Usage**: Right shift operations
- **Representation**: `let $out = $value >> $b` / `$value >>= $b`

---

### **Formatting Protocols**

#### **`Protocol::DISPLAY_FMT`**

- **Usage**: Display formatting
- **Representation**: `format!("{}", $value)`
- **Example**: Used in `println!` and template strings

#### **`Protocol::DEBUG_FMT`**

- **Usage**: Debug formatting
- **Representation**: `format!("{:?}", $value)`
- **Example**: Used for debug printing

---

### **Iterator Protocols**

#### **`Protocol::INTO_ITER`**

- **Usage**: Convert value into iterator
- **Representation**: `for item in $value { }`
- **Example**: Used in for loops

#### **`Protocol::NEXT`**

- **Usage**: Advance iteration
- **Method**: `next`
- **Representation**: `let $out = $value.next()`

#### **`Protocol::NEXT_BACK`**

- **Usage**: Advance iteration from back
- **Method**: `next_back`
- **Representation**: `let $out = $value.next_back()`

#### **`Protocol::NTH`**

- **Usage**: Jump to nth element
- **Method**: `nth`
- **Representation**: `let $out = $value.nth(index)`

#### **`Protocol::NTH_BACK`**

- **Usage**: Jump to nth element from back
- **Method**: `nth_back`

#### **`Protocol::SIZE_HINT`**

- **Usage**: Get iterator size hint
- **Method**: `size_hint`
- **Representation**: `let $out = $value.size_hint()`

#### **`Protocol::LEN`**

- **Usage**: Get length
- **Method**: `len`
- **Representation**: `let $out = $value.len()`

---

### **Special Protocols**

#### **`Protocol::INTO_FUTURE`**

- **Usage**: Convert value to awaitable future
- **Representation**: `$value.await`
- **Example**: Used with async/await

#### **`Protocol::TRY`**

- **Usage**: Question mark operator
- **Representation**: `value?`
- **Example**: Error handling with `?`
- **Demo**: See `/home/sid/rune/crates/rune/src/tests/vm_try.rs`

#### **`Protocol::CLONE`**

- **Usage**: Clone a value
- **Method**: `clone`
- **Representation**: `let $out = clone($value)`

#### **`Protocol::HASH`**

- **Usage**: Hash a value
- **Representation**: `let $out = hash($value)`

#### **`Protocol::INTO_TYPE_NAME`**

- **Usage**: Get type name as string
- **Example**: Type introspection

#### **`Protocol::IS_VARIANT`**

- **Usage**: Test if value is specific variant
- **Signature**: `fn(self, Hash) -> bool`
- **Example**: Enum variant checking

---

## Custom language server

From
https://github.com/VorpalBlade/paketkoll/blob/f1fc5823a8c9e8c3f12bbf95627598d0fad75758/crates/konfigkoll/src/bin/rune.rs

```rs
//! This is a helper binary for konfigkoll that provides Rune support functions
//! such as:
//!
//! * Documentation generation
//! * LSP language server
//! * Formatting of rune files
//! * Syntax checking
use konfigkoll_script::ScriptEngine;
#[cfg(target_env = "musl")]
use mimalloc::MiMalloc;

#[cfg(target_env = "musl")]
#[cfg_attr(target_env = "musl", global_allocator)]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    rune::cli::Entry::new()
        .about(format_args!("konfigkoll rune cli"))
        .context(&mut |_opts| ScriptEngine::create_context())
        .run();
}
```
