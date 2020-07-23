mod page_allocator;
pub mod page_tables;

use crate::peripherals::basic_fifo::BasicFIFO;
use core::fmt::Write;
use crate::mmu::page_allocator::{PageAllocator, PageRange};
use page_tables::PAGE_SIZE;
use crate::mmu::page_tables::{MMUManager, PageMapping};
use riscv::register;

extern "C" {
    // rx
    static _stext: u8;
    static _etext: u8;

    // r
    static _srodata: u8;
    static _erodata: u8;

    // These are together - rw
    static _sdata: u8;
    static _ebss: u8;

    // These are backwards, because heap - rw
    static _estack: u8;
    static _sstack: u8;

    fn _mret_direct();
}

pub fn setup_mmu() {
    let mut print_fifo = BasicFIFO::print_fifo();
    // grab our registers
    let (
        stext, etext,
        srodata, erodata,
        sdata, ebss,
        sstack, estack
    ) = unsafe {
        (
            &_stext as *const u8 as usize, &_etext as *const u8 as usize,
            &_srodata as *const u8 as usize, &_erodata as *const u8 as usize,
            &_sdata as *const u8 as usize, &_ebss as *const u8 as usize,
            &_sstack as *const u8 as usize, &_estack as *const u8 as usize
        )
    };

    // if debug, validate page align to make sure I didn't do something dumb in the linker
    if cfg!(debug_assertions) {
        // verify all our pages are aligned
        // Overlap is forbidden by the linker, since that would fail - so we don't have to check
        // Alignment will verify our pages don't overlap
        if (stext % PAGE_SIZE) != 0 {
            panic!("stext misaligned: {}", stext);
        }
        if (srodata % PAGE_SIZE) != 0 {
            panic!("srodata misaligned {}", srodata);
        }
        if (sdata % PAGE_SIZE) != 0 {
            panic!("sdata misaligned: {}", sdata);
        }
        if (estack % PAGE_SIZE) != 0 {
            panic!("Stack misaligned: {}", estack);
        }
    }

    // calculate page numbers
    let start_page = stext / PAGE_SIZE;
    // -1 means that if the page has zero bytes, it has zero pages
    let text_pages = PageRange { start: stext / PAGE_SIZE, end: ((etext - 1) / PAGE_SIZE) + 1 };
    let rodata_pages = PageRange { start: srodata / PAGE_SIZE, end: ((erodata - 1) / PAGE_SIZE) + 1 };
    let data_pages = PageRange { start: sdata / PAGE_SIZE, end: ((ebss - 1) / PAGE_SIZE) + 1 };
    let stack_pages = PageRange { start: estack / PAGE_SIZE, end: ((sstack - 1) / PAGE_SIZE) + 1 };

    let _ = writeln!(print_fifo, "text: {:x}..{:x}\nrodata: {:x}..{:x}\ndata: {:x}..{:x}\nstack: {:x}..{:x}",
                     text_pages.start, text_pages.end,
                     rodata_pages.start, rodata_pages.end,
                     data_pages.start, data_pages.end,
                     stack_pages.start, stack_pages.end);

    let max_page = stack_pages.end - 1;
    let _ = writeln!(print_fifo, "Highest current page: {:x}", max_page);

    // Create global page allocator
    let alloc_pages = PageAllocator::create_global(start_page, max_page);
    // The pages for the kernel have already been allocated

    // Allocate a page for the mmu manager
    if let Some(mmu_manager_page) = PageAllocator::get_global(|pg| pg.allocate()) {
        MMUManager::create_global(mmu_manager_page..(mmu_manager_page + 1))
    } else {
        panic!("Unable to allocate a page for the MMU manager")
    }

    // Map kernel space
    MMUManager::get_global(|mmu| {
        let kernel = mmu.get_kernel();
        for page in text_pages.start..text_pages.end {
            kernel.map_page(mmu, PageMapping {
                src: page,
                dest: page,
                user: false,
                read: true,
                write: false,
                execute: true,
            });
        }
        for page in rodata_pages.start..rodata_pages.end {
            kernel.map_page(mmu, PageMapping {
                src: page,
                dest: page,
                user: false,
                read: true,
                write: false,
                execute: false,
            });
        }
        for page in data_pages.start..data_pages.end {
            kernel.map_page(mmu, PageMapping {
                src: page,
                dest: page,
                user: false,
                read: true,
                write: true,
                execute: false,
            });
        }
        for page in stack_pages.start..stack_pages.end {
            kernel.map_page(mmu, PageMapping {
                src: page,
                dest: page,
                user: false,
                read: true,
                write: true,
                execute: false,
            });
        }
        for page in alloc_pages.start..alloc_pages.end {
            kernel.map_page(mmu, PageMapping {
                src: page,
                dest: page,
                user: false,
                read: true,
                write: true,
                execute: false,
            });
        }
    });

    // Turn on the MMU mapping
    MMUManager::get_global(|mmu| mmu.enable(mmu.kernel_space_id()));
    // Now we have to swap to supervisor mode
    unsafe {
        // First, set supervisor
        register::mstatus::set_mpp(register::mstatus::MPP::Supervisor);
        // Then call mret
        _mret_direct();
    }

    panic!("We probably forgot all our MMIO devices lmao")
}
