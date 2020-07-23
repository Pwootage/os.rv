use spin::RwLock;
use core::mem::size_of;
use core::ops::Range;
use crate::mmu::page_allocator::PageAllocator;
use crate::peripherals::basic_fifo::BasicFIFO;
use core::fmt::Write;
use riscv::register::satp;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_TABLE_SIZE: usize = PAGE_SIZE / 4;

bitfield! {
    pub struct PageTableEntry(u32);
    impl Debug;
    #[inline]
    pub ppn1, set_ppn1: 31, 20;
    #[inline]
    pub ppn0, set_ppn0: 19, 10;
    #[inline]
    pub ppn, set_ppn: 31, 10;
    #[inline]
    pub rsw, set_rsw: 9, 8;
    #[inline]
    pub d, set_d: 7;
    #[inline]
    pub a, set_a: 6;
    #[inline]
    pub g, set_g: 5;
    #[inline]
    pub u, set_u: 4;
    #[inline]
    pub x, set_x: 3;
    #[inline]
    pub w, set_w: 2;
    #[inline]
    pub r, set_r: 1;
    #[inline]
    pub v, set_v: 0;
}

pub type PageTable = [PageTableEntry; PAGE_TABLE_SIZE];

#[repr(C)]
pub struct VirtualMemorySpace {
    pub root_page: usize
}

#[derive(Copy, Clone, Debug)]
pub struct PageMapping {
    pub src: usize,
    pub dest: usize,
    pub user: bool,
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl VirtualMemorySpace {
    pub fn is_initialized(&self) -> bool {
        self.root_page != 0
    }

    /** Must run in machine mode */
    pub fn map_page(&self, mmu: &mut MMUManager, mapping: PageMapping) -> bool {
        let mut print_fifo = BasicFIFO::print_fifo();
        let _ = writeln!(print_fifo, "Request to map page {:x?}", mapping);

        let src_superpage = mapping.src / PAGE_TABLE_SIZE;
        let src_childpage = mapping.src % PAGE_TABLE_SIZE;
        let dest_superpage = mapping.dest / PAGE_TABLE_SIZE;
        let dest_childpage = mapping.dest % PAGE_TABLE_SIZE;

        let _ = writeln!(print_fifo, "Src: {:x}:{:x}", src_superpage, src_childpage);
        let _ = writeln!(print_fifo, "Dest: {:x}:{:x}", dest_superpage, dest_childpage);

        // Get the superpage
        let root_page_table = self.get_root_page_table();
        let superpage_entry = &mut root_page_table[src_superpage];

        // Get or allocate the child page offset
        let childpage_entry_page = if superpage_entry.v() {
            superpage_entry.ppn() as usize
        } else {
            if let Some(page) = PageAllocator::get_global(|pg| pg.allocate()) {
                superpage_entry.set_ppn(page as u32);
                superpage_entry.set_x(false);
                superpage_entry.set_r(false);
                superpage_entry.set_w(false);
                superpage_entry.set_u(true); // ignored, and subpages may be user-accessible
                superpage_entry.set_v(true);
                page
            } else {
                return false;
            }
        };
        // Get the child page and set it up
        let childpage_table = unsafe {
            &mut *((childpage_entry_page * PAGE_SIZE) as *mut PageTable)
        };
        let child_page_entry = &mut childpage_table[src_childpage];
        child_page_entry.set_ppn(mapping.dest as u32);
        child_page_entry.set_x(mapping.execute);
        child_page_entry.set_r(mapping.read);
        child_page_entry.set_w(mapping.write);
        child_page_entry.set_u(mapping.user);
        child_page_entry.set_v(true);

        return true;
    }

    fn get_root_page_table(&self) -> &mut PageTable {
        unsafe {
            return &mut *((self.root_page * PAGE_SIZE) as *mut PageTable);
        }
    }
}

#[repr(C)]
pub struct MMUManager {
    virtual_memory_spaces: *mut VirtualMemorySpace,
    kernel_id: Option<usize>,
    next_id: usize,
    max_spaces: usize,
}

static GLOBAL_MMU_MANAGER: RwLock<Option<MMUManager>> = RwLock::new(None);


impl MMUManager {
    pub fn create_global(memory_pages: Range<usize>) {
        let current = GLOBAL_MMU_MANAGER.upgradeable_read();
        if current.is_some() {
            panic!("Global mmu already initialized")
        }
        let mut write = current.upgrade();

        // size
        let space_size = size_of::<VirtualMemorySpace>();
        let max_spaces = PAGE_SIZE * memory_pages.len() / space_size;

        if max_spaces < 1 {
            panic!("Unable to allocate space for MMU managment! {}", max_spaces);
        }

        // Create page allocator
        let new = MMUManager {
            virtual_memory_spaces: (memory_pages.start * PAGE_SIZE) as *mut VirtualMemorySpace,
            kernel_id: None,
            next_id: 0,
            max_spaces,
        };

        // Zero out the memory
        unsafe {
            new.virtual_memory_spaces.write_bytes(0, max_spaces);
        }

        write.replace(new);
    }

    pub fn get_global<F, T>(f: F) -> T where F: Fn(&mut MMUManager) -> T {
        let mut lock = GLOBAL_MMU_MANAGER.write();
        let pg = lock.as_mut().unwrap();
        f(pg)
    }
}

impl MMUManager {
    fn get_space_raw<'a>(&self, id: usize) -> Option<&'a mut VirtualMemorySpace> {
        if id > self.max_spaces {
            return None;
        }
        unsafe {
            let page = self.virtual_memory_spaces.add(id);
            return Some(&mut *page);
        }
    }

    pub fn allocate_address_space(&mut self) -> Option<usize> {
        let asid = self.next_id;
        let space = self.get_space_raw(asid)?;
        if space.is_initialized() {
            return None;
        }
        // TODO: optimize?
        let mut next = self.next_id;
        loop {
            next += 1;
            if next > self.max_spaces || self.get_space_raw(next).is_none() {
                break;
            }
        }
        self.next_id = next;

        let page = PageAllocator::get_global(|pg| pg.allocate())?;
        space.root_page = page;

        return Some(asid);
    }

    pub fn get_space<'a>(&mut self, id: usize) -> Option<&'a mut VirtualMemorySpace> {
        let space = self.get_space_raw(id)?;
        if !space.is_initialized() {
            return None;
        }
        return Some(space);
    }

    pub fn get_kernel<'a>(&mut self) -> &'a mut VirtualMemorySpace {
        if let Some(kid) = self.kernel_id {
            return self.get_space(kid).unwrap();
        } else {
            let kid = self.allocate_address_space().unwrap();
            self.kernel_id = Some(kid);
            return self.get_space(kid).unwrap();
        }
    }

    pub fn kernel_space_id(&self) -> usize {
        return self.kernel_id.unwrap();
    }

    pub fn enable(&mut self, id: usize) {
        let space = self.get_space(id).unwrap();
        unsafe {
            satp::set(satp::Mode::Sv32, id, space.root_page);
        }
    }
}

// Mark it as send/sync (we are careful to make sure it's safe, since we are in charge of the pointer)
unsafe impl Sync for MMUManager {}

unsafe impl Send for MMUManager {}