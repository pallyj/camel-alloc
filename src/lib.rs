#![no_std]

#![feature(allocator_api)]
#![feature(let_else)]
#![feature(alloc_error_handler)]

//mod allocation;
mod chunk;
mod arena;
mod global;

pub use global::{init, memory_pressure};

use mu::{VmSpace, CapabilityBased, Credentials};
use spin::Mutex;

use core::alloc::Allocator;
use core::alloc::{Layout, AllocError};
use core::alloc::GlobalAlloc;
use core::sync::atomic::{AtomicUsize, Ordering};

use core::ptr::NonNull;

use arena::Arena;

const ARENAS: usize = 2;

pub struct CamelAllocator {
	arenas: [Mutex<Arena>; ARENAS],
	space: Mutex<Option<VmSpace>>,
	leaked: AtomicUsize,

}

impl CamelAllocator {
	pub const fn empty() -> CamelAllocator {
		CamelAllocator {
			arenas: [Mutex::new(Arena::new()),
					 Mutex::new(Arena::new())],
			space: Mutex::new(None),
			leaked: AtomicUsize::new(0)
		}
	}

	pub fn init(&self, space: &VmSpace) {
		let mut unlocked_space = self.space.lock();

		if unlocked_space.is_some() {
			panic!("ERROR: Can only initialize the allocator once");
		}

		*unlocked_space = Some(space.clone_capability(Credentials::all()).unwrap());

		for arena in self.arenas.iter() {
			arena.lock().use_space(unlocked_space.as_ref().unwrap());
		}
	}

	pub fn memory_used(&self) -> usize {
		self.arenas.iter()
			.map(|arena| arena.lock().used_size())
			.sum()
	}

	pub fn memory_allocated(&self) -> usize {
		self.arenas.iter()
			.map(|arena| arena.lock().size())
			.sum()
	}

	pub fn max_size(&self) -> usize {
		self.arenas.iter()
			.map(|arena| arena.lock().max_size())
			.sum()
	}

	pub fn leaked(&self) -> usize {
		self.leaked.load(Ordering::Relaxed)
	}
}

unsafe impl Allocator for CamelAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
		// Try to allocate from a non-locked arena
        for locked_arena in self.arenas.iter() {
			if locked_arena.is_locked() {
				continue;
			}

			let mut arena = locked_arena.lock();

			if let Some(allocation) = arena.alloc(layout) {
				return Ok(allocation);
			}
		}

		// If every arena is locked, or every non-locked arena is full,
		// Allocate by locking an arena
        for locked_arena in self.arenas.iter() {
			let mut arena = locked_arena.lock();

			if let Some(allocation) = arena.alloc(layout) {
				return Ok(allocation);
			}
		}

		Err(AllocError { })
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, layout: Layout) {
		let _leaked = self.leaked.fetch_add(layout.size(), Ordering::Relaxed);

		//rk::Debugger::printi(leaked);
    }
}

unsafe impl GlobalAlloc for CamelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		Allocator::allocate(&self, layout)
			.map_or(0 as *mut u8, |allocation| allocation.cast().as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		Allocator::deallocate(&self, NonNull::new_unchecked(ptr), layout)
    }
}