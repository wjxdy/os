use std::path::PathBuf;

fn main() {
    let out_dir =
        PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo did not provide OUT_DIR"));

    let kernel = PathBuf::from(
        std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel")
            .expect("Cargo did not provide the kernel binary artifact"),
    );

    let bios_path = out_dir.join("bios.img");
    bootloader::BiosBoot::new(&kernel)
        .create_disk_image(&bios_path)
        .expect("failed to create BIOS disk image");

    println!("cargo::rustc-env=BIOS_PATH={}", bios_path.display());
}
