use std::env;

fn main() {
    let bios_path = env!("BIOS_PATH");
    let kernel_binary: &str = env!("KERNEL_BINARY");

    println!("{}", kernel_binary);

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.arg("-drive")
        .arg(format!("format=raw,file={bios_path}"))
        .arg("-serial")
        .arg("mon:stdio");
        // .arg("-s")
        // .arg("-S");

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
