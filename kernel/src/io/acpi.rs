use core::ptr::NonNull;

use acpi::{
    AcpiTables, Handle, Handler, PciAddress, PhysicalMapping, aml::AmlError, platform::AcpiPlatform,
};
use spin::Once;
use x86_64::PhysAddr;

use crate::mem;

pub static ACPI_PLATFORM: Once<AcpiPlatform<AcpiHandler>> = Once::new();

/// # Safety
///
/// The caller must ensure that the rsdp_addr provided is valid
pub unsafe fn init(rsdp_addr: u64) {
    let acpi_data = unsafe { AcpiTables::from_rsdp(AcpiHandler, rsdp_addr as usize) }
        .expect("invalid rsdp _address");

    ACPI_PLATFORM.call_once(|| {
        AcpiPlatform::new(acpi_data, AcpiHandler).expect("platform provided invalid acpi data")
    });
}

#[derive(Clone)]
pub struct AcpiHandler;

impl Handler for AcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        // should already be mapped in page tables so another mapping is not needed

        let phys_addr = PhysAddr::new(physical_address as u64);
        let virt_addr = mem::phys_to_virt(phys_addr);

        PhysicalMapping {
            physical_start: physical_address,
            virtual_start: NonNull::new(virt_addr.as_mut_ptr())
                .expect("Failed to get virtual _address"),
            region_length: size,
            mapped_length: size,
            handler: self.clone(),
        }
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}

    fn read_u8(&self, _address: usize) -> u8 {
        unimplemented!()
    }

    fn read_u16(&self, _address: usize) -> u16 {
        unimplemented!()
    }

    fn read_u32(&self, _address: usize) -> u32 {
        unimplemented!()
    }

    fn read_u64(&self, _address: usize) -> u64 {
        unimplemented!()
    }

    fn write_u8(&self, _address: usize, _value: u8) {
        unimplemented!()
    }

    fn write_u16(&self, _address: usize, _value: u16) {
        unimplemented!()
    }

    fn write_u32(&self, _address: usize, _value: u32) {
        unimplemented!()
    }

    fn write_u64(&self, _address: usize, _value: u64) {
        unimplemented!()
    }

    fn read_io_u8(&self, _port: u16) -> u8 {
        unimplemented!()
    }

    fn read_io_u16(&self, _port: u16) -> u16 {
        unimplemented!()
    }

    fn read_io_u32(&self, _port: u16) -> u32 {
        unimplemented!()
    }

    fn write_io_u8(&self, _port: u16, _value: u8) {
        unimplemented!()
    }

    fn write_io_u16(&self, _port: u16, _value: u16) {
        unimplemented!()
    }

    fn write_io_u32(&self, _port: u16, _value: u32) {
        unimplemented!()
    }

    fn read_pci_u8(&self, _address: PciAddress, _offset: u16) -> u8 {
        unimplemented!()
    }

    fn read_pci_u16(&self, _address: PciAddress, _offset: u16) -> u16 {
        unimplemented!()
    }

    fn read_pci_u32(&self, _address: PciAddress, _offset: u16) -> u32 {
        unimplemented!()
    }

    fn write_pci_u8(&self, _address: PciAddress, _offset: u16, _value: u8) {
        unimplemented!()
    }

    fn write_pci_u16(&self, _address: PciAddress, _offset: u16, _value: u16) {
        unimplemented!()
    }

    fn write_pci_u32(&self, _address: PciAddress, _offset: u16, _value: u32) {
        unimplemented!()
    }

    fn nanos_since_boot(&self) -> u64 {
        unimplemented!()
    }

    fn stall(&self, _microseconds: u64) {
        unimplemented!()
    }

    fn sleep(&self, _milliseconds: u64) {
        unimplemented!()
    }

    fn create_mutex(&self) -> Handle {
        unimplemented!()
    }

    fn acquire(&self, _mutex: Handle, _timeout: u16) -> Result<(), AmlError> {
        unimplemented!()
    }

    fn release(&self, _mutex: Handle) {
        unimplemented!()
    }
}
