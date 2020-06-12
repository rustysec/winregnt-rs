extern crate winregnt;

use winregnt::RegKey;

fn main() {
    // Open the registry key
    let mut key = RegKey::open_write(r"\Registry\Machine\Software\DestroyMe").unwrap();

    key.write_dword_value("DwordValue", 1337)
        .expect("could not create dword value!");

    key.write_qword_value("QwordValue", 13371337)
        .expect("could not create qword value!");

    key.write_string_value("StringValue", "Hello, world!")
        .expect("could not create string value!");
}
