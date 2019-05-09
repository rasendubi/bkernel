use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    for script in &["stm32_flash.ld", "peripherals.ld", "isr_vector.ld"] {
        let mut f = File::open(script).expect(&format!("file {} not found", script));
        let mut contents = Vec::new();
        f.read_to_end(&mut contents).expect("read_to_end failed");
        File::create(out.join(script))
            .unwrap()
            .write_all(&contents)
            .unwrap();

        // Only re-run the build script when memory.ld is changed,
        // instead of when any part of the source code changes.
        println!("cargo:rerun-if-changed={}", script);
    }
    println!("cargo:rustc-link-search={}", out.display());
}
