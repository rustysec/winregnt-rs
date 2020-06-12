extern crate winregnt;

use winregnt::RegKey;

fn main() {
    // Open the registry key
    let key = RegKey::open_write(r"\Registry\Machine\Software\DestroyMe").unwrap();
    // Delete the value
    key.delete_value("DeleteThis")
        .expect("Couldn't delete the value");
}
