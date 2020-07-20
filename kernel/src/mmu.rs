use crate::peripherals::basic_fifo::BasicFIFO;
use core::fmt::Write;
use crate::peripherals::memory_size::MemorySize;
use crate::page_tables::{PAGE_SIZE, PageAllocator};

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
    let text_pages = (stext / PAGE_SIZE)..(((etext - 1) / PAGE_SIZE) + 1);
    let rodata_pages = (srodata / PAGE_SIZE)..(((erodata - 1) / PAGE_SIZE) + 1);
    let data_pages = (sdata / PAGE_SIZE)..(((ebss - 1) / PAGE_SIZE) + 1);
    let stack_pages = (estack / PAGE_SIZE)..(((sstack - 1) / PAGE_SIZE) + 1);

    let _ = writeln!(print_fifo, "text: {:x}..{:x}\nrodata: {:x}..{:x}\ndata: {:x}..{:x}\nstack: {:x}..{:x}",
                     text_pages.start, text_pages.end,
                     rodata_pages.start, rodata_pages.end,
                     data_pages.start, data_pages.end,
                     stack_pages.start, stack_pages.end);

    let max_page = stack_pages.end - 1;

    let _ = writeln!(print_fifo, "Highest current page: {:x}", max_page);

    let mem_size = MemorySize::new().max_size();
    let mem_pages = mem_size / PAGE_SIZE;
    let _ = writeln!(print_fifo, "Memory size: {:x}/{:x} pages", mem_size, mem_pages);
    // Create global page allocator
    PageAllocator::create_global(start_page, max_page + 1, mem_pages);

    // alloc/mark existing pages
    PageAllocator::get_global(|gp| {
        gp.allocate_range(&text_pages);
        gp.allocate_range(&rodata_pages);
        gp.allocate_range(&data_pages);
        gp.allocate_range(&stack_pages);
    })
}