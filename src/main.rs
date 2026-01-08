use std::{env, fs};

fn main() {
    let bios_path = env!("BIOS_PATH");
    let kernel_binary: &str = env!("KERNEL_BINARY");

    println!("{}", kernel_binary);

    let content = format!(
        r#"target create {kernel_binary}
target modules load --file {kernel_binary} --slide 0x10000000000
b kernel_start
gdb-remote localhost:1234
"#
    );
    fs::write(".lldbinit", content).expect("unable to create debug file");
    println!("debug file is ready, run `lldb -s debug.lldb` to start debugging");

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.arg("-drive")
        .arg(format!("format=raw,file={bios_path}"))
        .arg("-serial")
        .arg("mon:stdio");

    // cmd.arg("-s").arg("-S");
    cmd.arg("-d").arg("int").arg("-D").arg("qemu.log");

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
