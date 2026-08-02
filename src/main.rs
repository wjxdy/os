use std::process::Command;

fn main() {
    let bios_path = env!("BIOS_PATH");
    let drive = format!("format=raw,file={bios_path}");

    let status = Command::new("qemu-system-x86_64")
        .arg("-accel")
        .arg("tcg")
        .arg("-drive")
        .arg(drive)
        .arg("-serial")
        .arg("mon:stdio")
        .arg("-display")
        .arg("none")
        .arg("-no-reboot")
        .status()
        .expect("failed to start qemu-system-x86_64");

    assert!(status.success(), "QEMU exited with status {status}");
}
