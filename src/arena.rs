use core::{alloc::Layout, ptr::NonNull};

use arrayvec::ArrayVec;
use mu::VmSpace;
use crate::chunk::Chunk;

// Allows for 2 MB of total heap
const CHUNK_SIZE: usize = 2 * 1024 * 1024;
const CHUNK_CAPACITY: usize = 16;

pub struct Arena {
	chunks: ArrayVec<Chunk, CHUNK_CAPACITY>,
	size: usize,
	head: usize,
	space: Option<VmSpace>,
	//TODO: Refactor this
	temporary_space: [u8; 0x1000],
	temporary_ptr: usize,
}

impl Arena {
	pub const fn new() -> Self {
		Arena {
			chunks: ArrayVec::new_const(),
			size: 0,
			head: 0,
			space: None,
			temporary_space: [0u8; 0x1000],
			temporary_ptr: 0
		}
	}

	pub fn use_space(&mut self, space: &VmSpace) {
		self.space = Some(*space.clone());
	}

	#[allow(dead_code)]
	pub fn used_size(&self) -> usize {
		self.head
	}

	#[allow(dead_code)]
	pub fn free_size(&self) -> usize {
		self.max_size() - self.head
	}

	#[allow(dead_code)]
	pub fn size(&self) -> usize {
		self.size
	}

	#[allow(dead_code)]
	pub fn max_size(&self) -> usize {
		CHUNK_CAPACITY * CHUNK_SIZE
	}

	pub fn alloc(&mut self, layout: Layout) -> Option<NonNull<[u8]>> {
		self.align_to(layout.align());

		let mut allocation = None;

		for chunk in self.chunks.iter() {
			// Check if the chunk holds the pointer
			if chunk.contains(self.head) {
				// Check if the chunk has enough space to hold the layout
				if let Some(ptr) = chunk.try_get_slice(self.head, layout.size()) {
					// Choose this chunk

					self.head += layout.size();
					allocation = Some(ptr);
				}

				break;
			}
		}

		let Some(allocated) = allocation else {
			match self.extend() {
				Some(_) => return self.alloc(layout),
				None => {
					if (0x1000 - self.temporary_ptr) < layout.size() {
						return None;
					}

					let ptr = unsafe { NonNull::new_unchecked(&mut self.temporary_space[self.temporary_ptr..(self.temporary_ptr + layout.size())]) };

					self.temporary_ptr += layout.size();

					return Some(ptr)
				}
			}
		};

		return Some(allocated);
	}

	fn align_to(&mut self, align: usize) {
		self.head = (self.head & !(align - 1)) + align;
	}

	fn extend(&mut self) -> Option<()> {
		// If the allocator hasn't been initialized, don't panic
		if self.space.is_none() {
			return None
		}

		// Don't bother allocating a new chunk if the arena is full
		if self.chunks.is_full() {
			return None
		}

		// TODO: Test if the chunk was allocated
		// For now, just panic if it couldn't be allocated
		let new_chunk = Chunk::new(self.size, CHUNK_SIZE, self.space.as_ref().unwrap());

		// Update the arena with the new chunk
		self.head = self.size;
		self.chunks.push(new_chunk);
		self.size += CHUNK_SIZE;


		Some(())
	}
}