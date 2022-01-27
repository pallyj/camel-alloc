use core::ptr::NonNull;

use mu::{VmObject, VAddr, MemoryFlags, VmSpace, MappingFlags, println};

#[allow(dead_code)]
pub struct Chunk {
	base: usize,

	start: VAddr,
	size: usize,

	backing: VmObject
}

impl Chunk {
	pub fn new(base: usize, size: usize, space: &VmSpace) -> Self {
		let backing = VmObject::new(size, MemoryFlags::RW).unwrap();

		let start = match space.map(0, &backing, 0, size, MappingFlags::RW) {
			Ok(a) => a,
			Err(e) => {mu::dev::debugger::print_status(e); loop{}},
		};

		Self {
			base,

			start,
			size,

			backing
		}
	}

	pub fn try_get_slice(&self, addr: usize, size: usize) -> Option<NonNull<[u8]>> {
		if addr < self.base {
			return None
		}

		let offset = addr - self.base;

		if (offset + size) >= self.size {
			return None
		}

		NonNull::new(unsafe { (self.start + offset).into_slice_mut(size) })
	}

	pub fn contains(&self, addr: usize) -> bool {
		if addr < self.base {
			return false
		}

		let offset = addr - self.base;

		if offset >= self.size {
			return false
		}
		
		return true
	}
}

/*impl Drop for Chunk {
	pub fn drop(&mut self) {
		self.backing.drop()
	}
}*/