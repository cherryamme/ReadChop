//! Priority: READCHOP_PASSPHRASE env > /etc/machine-id > random 26-char string

use std::env;

fn main() {
    let passphrase = get_passphrase();
    println!("cargo:rustc-env=READCHOP_PASSPHRASE={}", passphrase);
}

fn get_passphrase() -> String {
    // 1. Environment variable
    if let Ok(p) = env::var("READCHOP_PASSPHRASE") {
        if !p.is_empty() {
            return p;
        }
    }

    // 2. /etc/machine-id
    if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
        let id = id.trim();
        if !id.is_empty() {
            return id.to_string();
        }
    }

    // 3. Random 26-char string from /dev/urandom
    use std::io::Read;
    let mut file = std::fs::File::open("/dev/urandom").unwrap();
    let mut bytes = [0u8; 26];
    file.read_exact(&mut bytes).unwrap();
    bytes.iter().map(|b| {
        let idx = (*b % 52) as u8;
        if idx < 26 { (b'A' + idx) as char } else { (b'a' + idx - 26) as char }
    }).collect()
}
