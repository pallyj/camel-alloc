use mu::VmSpace;

use crate::CamelAllocator;

#[global_allocator]
static ALLOC: CamelAllocator = CamelAllocator::empty();

pub fn init(handle: &VmSpace) {
	ALLOC.init(handle);
}

pub struct MemoryPressure {
	pub used: usize,
	pub size: usize,
	pub available: usize,
	pub leaked: usize
}

pub fn memory_pressure() -> MemoryPressure {
	let used = ALLOC.memory_used();
	let size = ALLOC.memory_allocated();
	let available = ALLOC.max_size();
	let leaked = ALLOC.leaked();

	MemoryPressure {
		used,
		size,
		available,
		leaked
	}
}

#[alloc_error_handler]
pub fn handle_alloc_error(_layout: core::alloc::Layout) -> ! {
	panic!("Error Allocating");
}