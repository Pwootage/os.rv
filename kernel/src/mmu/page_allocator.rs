use crate::peripherals::memory_size::MemorySize;
use bitvec::prelude::*;
use bitvec::slice::bits_from_raw_parts_mut;
use crate::mmu::page_tables::PAGE_SIZE;
use bitvec::indices::BitIdx;
use spin::RwLock;

#[repr(C)]
pub struct PageAllocator<'a> {
    total_pages: usize,
    page_base: usize,
    alloc_table: &'a mut BitSlice<Msb0, u8>,
    next_open_page: usize,
}

pub struct PageRange { pub start: usize, pub end: usize }

static GLOBAL_PAGE_ALLOCATOR: RwLock<Option<PageAllocator>> = RwLock::new(None);

impl PageAllocator<'_> {
    pub fn create_global(base_page: usize, max_page: usize) -> PageRange {
        let current = GLOBAL_PAGE_ALLOCATOR.upgradeable_read();
        if current.is_some() {
            panic!("Global page allocator already initialized")
        }
        let mut write = current.upgrade();

        // Figure out size of main ram
        let mem_size = MemorySize::new().max_size();
        let mem_pages = mem_size / PAGE_SIZE;

        // Figure out how many pages of bits we need to track allocation status
        let table_size = mem_pages / 8 / PAGE_SIZE + if mem_pages % (8 * PAGE_SIZE) > 0 { 1 } else { 0 };
        let alloc_pages = PageRange { start: max_page + 1, end: max_page + table_size + 1 };

        // Create page allocator
        let mut new = PageAllocator {
            page_base: base_page,
            total_pages: mem_pages,
            alloc_table: unsafe {
                bits_from_raw_parts_mut(
                    (alloc_pages.start * PAGE_SIZE) as *mut u8,
                    BitIdx::new(0).unwrap(),
                    mem_pages,
                )
            },
            next_open_page: alloc_pages.end,
        };

        // Zero out the memory
        new.alloc_table.set_all(false);

        for page in base_page..alloc_pages.end {
            new.write_bit(page, true)
        }

        write.replace(new);

        return alloc_pages;
    }

    pub fn get_global<F, T>(f: F) -> T where F: Fn(&mut PageAllocator) -> T {
        let mut lock = GLOBAL_PAGE_ALLOCATOR.write();
        let pg = lock.as_mut().unwrap();
        f(pg)
    }
}

impl PageAllocator<'_> {
    #[inline]
    fn page_entry(&self, page: usize) -> usize {
        return page - self.page_base;
    }

    fn read_bit(&self, page: usize) -> bool {
        let entry = self.page_entry(page);
        if entry > self.total_pages {
            return true;
        }
        self.alloc_table[entry]
    }

    fn write_bit(&mut self, page: usize, value: bool) {
        let entry = self.page_entry(page);
        if entry > self.total_pages {
            return;
        }
        self.alloc_table.set(entry, value)
    }

    pub fn is_allocated(&self, page: usize) -> bool {
        return self.read_bit(page);
    }

    pub fn allocate(&mut self) -> Option<usize> {
        if self.page_entry(self.next_open_page) > self.total_pages {
            None
        } else {
            let page = self.next_open_page;
            // find next open page
            // TODO: optimize
            loop {
                self.next_open_page += 1;
                if self.page_entry(self.next_open_page) > self.total_pages || !self.read_bit(self.next_open_page) {
                    break;
                }
            }

            self.write_bit(page, true);

            // Zero the page
            let ptr = (page * PAGE_SIZE) as *mut u8;
            unsafe {
                ptr.write_bytes(0, PAGE_SIZE);
            }

            Some(page)
        }
    }

    pub fn deallocate(&mut self, page: usize) {
        if page < self.next_open_page {
            self.next_open_page = page;
        }
        self.write_bit(page, false);
    }
}

// Mark it as send/sync (we are careful to make sure it's safe, since we are in charge of the pointer)
unsafe impl Sync for PageAllocator<'_> {}

unsafe impl Send for PageAllocator<'_> {}