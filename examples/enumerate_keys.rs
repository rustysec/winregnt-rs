extern crate winregnt;

use winregnt::RegKey;

fn main() {
    let reg =
        RegKey::open(r"\Registry\Machine\Software\Microsoft\Windows\CurrentVersion".to_owned())
            .unwrap();
    println!("Keys:");
    reg.enum_keys().for_each(|k| {
        println!("- {}", k);
        k.open().map(|key| {
            key.enum_values()
                .for_each(|v| println!("-- {}: {}", v, v.value()));
        });
    });
}
