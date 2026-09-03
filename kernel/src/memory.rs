use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::structures::paging::PageTable;
use x86_64::structures::paging::{
    FrameAllocator, OffsetPageTable, PageSize, PhysFrame, Size4KiB, Translate,
    mapper::TranslateResult,
};

use x86_64::{PhysAddr, VirtAddr, registers::control::Cr3};

pub unsafe fn init_offset_page_table(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = unsafe { active_level_4_table(physical_memory_offset) };
    unsafe { OffsetPageTable::new(level_4_table, physical_memory_offset) }
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();
    let physical_address = level_4_table_frame.start_address();
    let virtual_address = physical_memory_offset + physical_memory_offset.as_u64();
    let page_table_pointer: *mut PageTable = virtual_address.as_mut_ptr();

    unsafe { &mut *page_table_pointer }
}

pub struct BootInfoFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_regions: &'static MemoryRegions) -> Self {
        Self {
            memory_regions,
            next: 0,
        }
    }
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame<Size4KiB>> + '_ {
        self.memory_regions
            .iter()
            .filter(|region| region.kind == MemoryRegionKind::Usable)
            .flat_map(|region| {
                let start = PhysAddr::new(region.start)
                    .align_up(Size4KiB::SIZE)
                    .as_u64();

                let end = PhysAddr::new(region.end)
                    .align_down(Size4KiB::SIZE)
                    .as_u64();

                (start..end).step_by(Size4KiB::SIZE as usize)
            })
            .map(|address| {
                PhysFrame::from_start_address(PhysAddr::new(address))
                    .expect("aligned usable frame address")
            })
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next = self.next.saturating_add(1);
        frame
    }
}

pub fn print_memory_map(memory_regions: &MemoryRegions) {
    crate::println!("PHYSICAL MEMORY MAP");

    for (index, region) in memory_regions.iter().enumerate() {
        crate::println!(
            "REGION {index}: {:#x}..{:#x} {:?}",
            region.start,
            region.end,
            region.kind,
        )
    }
}
