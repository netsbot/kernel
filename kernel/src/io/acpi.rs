use acpi::{Handler, PciAddress, PhysicalMapping};
use core::ptr;
use core::ptr::NonNull;
use x86_64::{PhysAddr, VirtAddr};

#[derive(Clone)]
pub struct AcpiHandler {
    physical_memory_offset: VirtAddr,
}

impl AcpiHandler {
    pub fn new(physical_memory_offset: VirtAddr) -> Self {
        Self {
            physical_memory_offset,
        }
    }

    fn read_addr<T>(&self, addr: usize) -> T
    where
        T: Copy,
    {
        let phys_addr = PhysAddr::new(addr as u64);
        let virt_addr = self.physical_memory_offset + phys_addr.as_u64();
        unsafe { ptr::read_volatile(virt_addr.as_ptr::<T>()) }
    }
}

impl Handler for AcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let phys_addr = PhysAddr::new(physical_address as u64);
        let virt_addr = self.physical_memory_offset + phys_addr.as_u64();

        PhysicalMapping {
            physical_start: physical_address,
            virtual_start: NonNull::new(virt_addr.as_mut_ptr())
                .expect("Failed to get virtual address"),
            region_length: size,
            mapped_length: size,
            handler: self.clone(),
        }
    }

    fn unmap_physical_region<T>(region: &PhysicalMapping<Self, T>) {}

    fn read_u8(&self, address: usize) -> u8 {
        Self::read_addr(self, address)
    }

    fn read_u16(&self, address: usize) -> u16 {
        Self::read_addr(self, address)
    }

    fn read_u32(&self, address: usize) -> u32 {
        Self::read_addr(self, address)
    }

    fn read_u64(&self, address: usize) -> u64 {
        Self::read_addr(self, address)
    }

    fn write_u8(&self, address: usize, value: u8) {
        unimplemented!()
    }

    fn write_u16(&self, address: usize, value: u16) {
        unimplemented!()
    }

    fn write_u32(&self, address: usize, value: u32) {
        unimplemented!()
    }

    fn write_u64(&self, address: usize, value: u64) {
        unimplemented!()
    }

    fn read_io_u8(&self, port: u16) -> u8 {
        unimplemented!()
    }

    fn read_io_u16(&self, port: u16) -> u16 {
        unimplemented!()
    }

    fn read_io_u32(&self, port: u16) -> u32 {
        unimplemented!()
    }

    fn write_io_u8(&self, port: u16, value: u8) {
        unimplemented!()
    }

    fn write_io_u16(&self, port: u16, value: u16) {
        unimplemented!()
    }

    fn write_io_u32(&self, port: u16, value: u32) {
        unimplemented!()
    }

    fn read_pci_u8(&self, address: PciAddress, offset: u16) -> u8 {
        unimplemented!()
    }

    fn read_pci_u16(&self, address: PciAddress, offset: u16) -> u16 {
        unimplemented!()
    }

    fn read_pci_u32(&self, address: PciAddress, offset: u16) -> u32 {
        unimplemented!()
    }

    fn write_pci_u8(&self, address: PciAddress, offset: u16, value: u8) {
        unimplemented!()
    }

    fn write_pci_u16(&self, address: PciAddress, offset: u16, value: u16) {
        unimplemented!()
    }

    fn write_pci_u32(&self, address: PciAddress, offset: u16, value: u32) {
        unimplemented!()
    }

    fn nanos_since_boot(&self) -> u64 {
        unimplemented!()
    }

    fn stall(&self, microseconds: u64) {
        unimplemented!()
    }

    fn sleep(&self, milliseconds: u64) {
        unimplemented!()
    }
}
