use x86_64::instructions::port::Port;

pub fn disable_pic() {
    let mut master_pic = Port::new(0x21);
    let mut slave_pic = Port::new(0xa1);

    unsafe {
        master_pic.write(0xffu8);
        slave_pic.write(0xffu8);
    }
}
