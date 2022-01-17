use core::cmp::max;

const QUANTUM: usize = 16;
const PAGE_SZ: usize = 4096;

pub enum AllocationSize {
	Tiny(usize), // power of two, 2, 4, 8
	Quantum(usize), // Multiple of a quantum
	Kilo(usize), // power of 2 kilobytes
	Mega(usize), // Multiple of 2 MB
}

pub enum SizeCategory {
	Small,
	Large,
	Huge
}

impl AllocationSize {
	pub fn size_of(size: usize) -> Self {
		if size <= (QUANTUM / 2) {
			let pow = size.next_power_of_two();
			AllocationSize::Tiny(max(pow, 2))
		} else if size <= 512 {
			AllocationSize::Quantum(size / QUANTUM)
		} else if size <= (1024 * 1024) {
			let pow = size.next_power_of_two();
			AllocationSize::Kilo(pow - 10)
		} else {
			AllocationSize::Mega(size / (2 * 1024 * 1024))
		}
	}
	
	pub fn size(&self) -> usize {
		match self {
			Self::Tiny(pow_2) => 1 << pow_2,
			Self::Quantum(multiple) => multiple * QUANTUM,
			Self::Kilo(pow_2) => 1024 << pow_2,
			Self::Mega(multiple) => 2 * 1024 * 1024 * multiple,
		}
	}

	pub fn category(&self) -> SizeCategory {
		match self {
			Self::Tiny(_) => SizeCategory::Small,
			Self::Quantum(_) => SizeCategory::Small,
			Self::Kilo(pow_2) => {
				if (1024 << pow_2) < PAGE_SZ {
					SizeCategory::Small
				} else {
					SizeCategory::Large
				}
			}
			Self::Mega(_) => SizeCategory::Huge,
		}
	}
}