extern crate winregnt;

use winregnt::RegKey;

fn main() {
    let key = RegKey::open_write(r"\Registry\Machine\Software\DestroyMe").unwrap();
    key.delete().expect("Couldn't delete the key");
}
