use spin::RwLock;
use core::ops::Range;

pub const PAGE_SIZE: usize = 4096;

bitfield! {
    pub struct PageTableEntry(u32);
    impl Debug;
    #[inline]
    pub ppn1, set_ppn1: 31, 20;
    #[inline]
    pub ppn0, set_ppn0: 19, 10;
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

pub type PageTable = [PageTableEntry; PAGE_SIZE];

pub struct VirtualMemorySpace {
    pub asid: u32
}

#[repr(C)]
pub struct PageAllocator {
    total_pages: usize,
    page_base: usize,
    alloc_table: *mut u8,
}

impl PageAllocator {
    pub fn create_global(base_page: usize, alloc_page: usize, total_pages: usize) {
        let current = GLOBAL_PAGE_ALLOCATOR.upgradeable_read();
        if current.is_some() {
            panic!("Global page allocator already initialized")
        }
        let mut write = current.upgrade();
        let mut new = PageAllocator {
            page_base: base_page,
            total_pages,
            alloc_table: (alloc_page * PAGE_SIZE) as *mut u8,
        };

        let table_size = total_pages / 8 / PAGE_SIZE + if total_pages % (8 * PAGE_SIZE) > 0 { 1 } else { 0 };

        let alloc_pages = alloc_page..(alloc_page + table_size);
        if !new.allocate_range(&alloc_pages) {
            panic!("Failed to allocate pages for page table")
        }

        write.replace(new);
    }

    pub fn get_global<F>(f: F) where F: Fn(&mut PageAllocator) -> () {
        let mut lock = GLOBAL_PAGE_ALLOCATOR.write();
        let pg = lock.as_mut().unwrap();
        f(pg)
    }
}

impl PageAllocator {
    fn page_entry(&self, page: usize) -> usize {
        return page - self.page_base;
    }

    fn read_bit(&self, page: usize) -> bool {
        let entry = self.page_entry(page);
        if entry > self.total_pages {
            return true;
        }
        let byte = entry / 8;
        let bit = entry % 8;
        unsafe {
            let b = *self.alloc_table.add(byte);
            ((b >> bit as u8) & 1) == 1
        }
    }

    fn write_bit(&mut self, page: usize, value: bool) {
        let entry = self.page_entry(page);
        if entry > self.total_pages {
            return;
        }
        let byte = entry / 8;
        let bit = entry % 8;
        unsafe {
            let bp = self.alloc_table.add(byte);
            if value {
                *bp = (*bp) | (1 << bit) as u8;
            } else {
                *bp = (*bp) | (!(1 << bit)) as u8;
            }
        }
    }

    pub fn is_allocated(&self, page: usize) -> bool {
        return self.read_bit(page);
    }

    pub fn allocate(&mut self, page: usize) -> bool {
        if self.read_bit(page) {
            return false;
        } else {
            self.write_bit(page, true);
            return true;
        }
    }
    pub fn deallocate(&mut self, page: usize) -> bool {
        if !self.read_bit(page) {
            return false;
        } else {
            self.write_bit(page, false);
            return true;
        }
    }

    pub fn allocate_range(&mut self, pages: &Range<usize>) -> bool {
        // TODO: optimize, maybe
        let mut last_page: Option<usize> = None;
        for page in pages.start..pages.end {
            if !self.allocate(page) {
                last_page = Some(page);
                break;
            }
        }
        if let Some(end) = last_page {
            for page in pages.start..end {
                self.deallocate(page);
            }
            false
        } else {
            true
        }
    }
}

static GLOBAL_PAGE_ALLOCATOR: RwLock<Option<PageAllocator>> = RwLock::new(None);

// Mark it as send/sync (we are careful to make sure it's safe, since we are in charge of the pointer)
unsafe impl Sync for PageAllocator {}

unsafe impl Send for PageAllocator {}