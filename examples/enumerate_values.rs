extern crate winregnt;

use winregnt::RegKey;

fn main() {
    let reg =
        RegKey::open(r"\Registry\Machine\Software\Microsoft\Windows\CurrentVersion\Run".to_owned())
            .unwrap();
    println!("Values:");
    reg.enum_values().for_each(|k| {
        println!("- {}: {}", k, k.value());
    });
}
