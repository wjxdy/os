use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::structures::paging::{
    FrameAllocator, OffsetPageTable, PageSize, PhysFrame, Size4KiB, Translate,
    mapper::TranslateResult,
};
use x86_64::structures::paging::{OffsetPageTable, PageTable, PageTableFlags};

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

pub fn print_address_pasts(label: &str, address: VirtAddr) {
    let p4: u16 = address.p4_index().into();
    let p3: u16 = address.p3_index().into();
    let p2: u16 = address.p2_index().into();
    let p1: u16 = address.p1_index().into();
    let offset: u16 = address.page_offset().into();

    crate::println!("{label} PARTS: p4={p4}, p3={p3}, p2={p2}, p1={p1}, offset_4k={offset:#x}");
}

pub fn print_translation(mapper: &OffsetPageTable<'_>, name: &str, address: VirtAddr) {
    match mapper.translate(address) {
        TranslateResult::Mapped {
            frame,
            offset,
            flags,
        } => {
            let physical_address = frame.start_address() + offset;
            crate::println!(
                "TRANSLATE {name}, virt={:#x} -> phys={:#x}, frame={:?}, offset={:#x}, leaf_flags={:?}",
                address.as_u64(),
                physical_address.as_u64(),
                frame,
                offset,
                flags,
            );
        }
        TranslateResult::NotMapped => {
            crate::println!(
                "TRASNSLATE {name}: virt={:#x} -> NOT MAPPAD",
                address.as_u64()
            );
        }
        TranslateResult::InvalidFrameAddress(physical_address) => {
            crate::println!(
                "TANSLATE {name}: virt={:#x} -> INVALID FRAME ADDRESS {:#x}",
                address.as_u64(),
                physical_address.as_u64(),
            );
        }
    }
}

pub struct BootInfoFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
}

pub unsafe fn init_offset_page_table(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = unsafe { active_level_4_table(physical_memory_offset) };

    unsafe { OffsetPageTable::new(level_4_table, physical_memory_offset) }
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();
    let physical_address = level_4_table_frame.start_address();
    let virtual_address = physical_memory_offset + physical_address.as_u64();
    let page_table_pointer: *mut PageTable = virtual_address.as_mut_ptr();

    unsafe { &mut *page_table_pointer }
}

pub fn print_level_4_entry(mapper: &OffsetPageTable<'_>, address: VirtAddr) {
    let index = address.p4_index();
    let index_number: u16 = index.into();
    let entry = &mapper.level_4_table()[index];

    if !entry.flags().contains(PageTableFlags::PRESENT) {
        crate::println!("P4 ENTRY {index_number}: NOT PRESENT");
        return;
    }

    crate::println!(
        "P4 ENTRY {index_number}: next_table_frame={:#x}, flags={:?}",
        entry.addr().as_u64(),
        entry.flags(),
    );
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
