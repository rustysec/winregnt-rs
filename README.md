# WinRegNt
A Rust interface to `Nt*` series of windows registry APIs. [winreg](https://github.com/gentoo90/winreg-rs) is a
fantastic library but uses the common (and friendly) win32 APIs to interact with the registry. This leaves
some blind spots when dealing with `null` characters which are permitted by the `Nt` functions and _not_ by
Win32. Some information about this can be found 
[here](https://docs.microsoft.com/en-us/sysinternals/downloads/reghide).

This is a _work in progress_ and does not support all registry features!

## Usage
In your `cargo.toml`:

```toml
winregnt = { git = "https://github.com/rustysec/winregnt-rs" }
```

`main.rs`:

```rust
use winregnt::RegKey;

fn main() {
    let key = RegKey::open(r"\Registry\Users").unwrap();
    key.enum_keys().for_each(|k| println!("- {}", k));
}
```

