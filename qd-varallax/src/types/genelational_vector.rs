//構造体
//Generation Index
#[derive(Hash, Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct VxGenIndex {
	pub(crate) index: usize,
	pub(crate) generation: u64,
}

//Slot
pub(crate) enum VxSlot<T> {
	Using {
		data: T,
		generation: u64,
	},
	Free {
		next_free: Option<usize>,
		generation: u64,
	}
}

//Generation Iterator
pub struct GenIter<'a, T> {
	inner: std::slice::Iter<'a, VxSlot<T>>,
}

//Generation Iterator (mut)
pub struct GenIterMut<'a, T> {
	inner: std::slice::IterMut<'a, VxSlot<T>>,
}

//Generation Iterator and ID
pub struct GenIterWithId<'a, T> {
	inner: std::iter::Enumerate<std::slice::Iter<'a, VxSlot<T>>>,
}

//Generation Iterator and ID (mut)
pub struct GenIterWithIdMut<'a, T> {
	inner: std::iter::Enumerate<std::slice::IterMut<'a, VxSlot<T>>>,
}

//Generation Vector
pub struct VxGenVector<T> {
	pub(crate) slots: Vec<VxSlot<T>>,
	free_head: Option<usize>,
	len: usize,
}

//実装---------------------------------------------------------------

//Generation Index
impl VxGenIndex {
	pub(crate) fn new(index: usize, generation: u64) -> Self {
		Self {
			index,
			generation,
		}
	}
}

//Generation Iterator
impl <'a, T> Iterator for GenIter<'a, T> {
	type Item = &'a T;
	
	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.inner.next()? {
				VxSlot::Using { data, .. } => return Some(data),
				VxSlot::Free { .. } => continue,
			}
		}
	}
}

//Generation Iterator (mut)
impl <'a, T> Iterator for GenIterMut<'a, T> {
	type Item = &'a mut T;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.inner.next()? {
				VxSlot::Using { data, .. } => return Some(data),
				VxSlot::Free { .. } => continue,
			}
		}
	}
}

//Generation Iterator and ID
impl <'a, T> Iterator for GenIterWithId<'a, T> {
	type Item = (VxGenIndex, &'a T);

	fn next(&mut self) -> Option<Self::Item> {
		while let Some((index, slot)) = self.inner.next() {
			if let VxSlot::Using { data, generation } = slot {
				return Some((
					VxGenIndex::new(index, *generation),
					data
				));
			}
		}
		None
	}
}

//Generation Iterator and ID (mut)
impl <'a, T> Iterator for GenIterWithIdMut<'a, T> {
	type Item = (VxGenIndex, &'a mut T);

	fn next(&mut self) -> Option<Self::Item> {
		while let Some((index, slot)) = self.inner.next() {
			if let VxSlot::Using { data, generation } = slot {
				return Some((
					VxGenIndex::new(index, *generation),
					data
				));
			}
		}
		None
	}
}

//Generation Vector Index trait
impl<T> std::ops::Index<VxGenIndex> for VxGenVector<T> {
	type Output = T;

	#[inline]
	fn index(&self, index: VxGenIndex) -> &Self::Output {
		self.get(index).expect("Invalid Generation Index")
	}
}

//Generation Vector
impl <T> VxGenVector<T> {
	#[inline]
	pub const fn new() -> Self {
		Self {
			slots: Vec::new(),
			free_head: None,
			len: 0,
		}
	}

	//controll
	pub fn insert(&mut self, data: T) -> VxGenIndex {
		if let Some(index) = self.free_head {
			match &mut self.slots[index] {
				VxSlot::Free { next_free, generation } => {
					let g = *generation;
					self.free_head = *next_free;
					
					self.slots[index] = VxSlot::Using { data, generation: g };
					self.len += 1;
					VxGenIndex::new(index, g)
				}
				_ => unreachable!("free_head pointed to a Using slot")
			}
		} else {
			let index = self.slots.len();
			let generation = 0;
			self.slots.push(VxSlot::Using {
				data,
				generation, 
			});
			self.len += 1;
			VxGenIndex::new(index, generation)
		}
	}

	pub fn remove(&mut self, id: VxGenIndex) -> Option<T> {
		let slot = self.slots.get_mut(id.index)?;

		match slot {
			VxSlot::Using { generation , .. } if *generation == id.generation => {
				let old_gen= *generation;
				let old_slot = std::mem::replace(
					slot,
					VxSlot::Free {
						next_free: self.free_head,
						generation: old_gen + 1
					}
				);
				if let VxSlot::Using { data, .. } = old_slot {
					self.free_head = Some(id.index);
					self.len -= 1;
					return Some(data);
				}
				None
			}
			_ => None,
		}
	}

	//getter
	pub fn get(&self, id: VxGenIndex) -> Option<&T> {
		match self.slots.get(id.index)? {
			VxSlot::Using {
				data,
				generation
			} if *generation == id.generation => Some(data),
			_ => None,
		}
	}
	pub fn get_mut(&mut self, id: VxGenIndex) -> Option<&mut T> {
		match self.slots.get_mut(id.index)? {
			VxSlot::Using {
				data,
				generation
			} if *generation == id.generation => Some(data),
			_ => None,
		}
	}
	pub fn get_two_mut(&mut self, id1: VxGenIndex, id2: VxGenIndex) -> Option<(&mut T, &mut T)> {
		if id1.index == id2.index { return None; }
		
		let (min_index, max_index, swapped) = if id1.index < id2.index {
			(id1, id2, false)
		} else {
			(id2, id1, true)
		};

		let (first, second) = self.slots.split_at_mut(min_index.index + 1);

		let slot_min = &mut first[min_index.index];
		let slot_max = &mut second[max_index.index - (min_index.index + 1)];

		match (slot_min, slot_max) {
			(
				VxSlot::Using { data: d_min, generation: g_min },
				VxSlot::Using { data: d_max, generation: g_max }
			) if *g_min == min_index.generation && *g_max == max_index.generation => {
				if swapped {
					Some((d_max, d_min))
				} else {
					Some((d_min, d_max))
				}
			}
			_ => None,
		}
	}

	pub fn last(&self) -> Option<&T> {
		self.slots.iter().rev().find_map(|slot| {
			if let VxSlot::Using { data, .. } = slot {
				Some(data)
			} else {
				None
			}
		})
	}

	pub fn last_mut(&mut self) -> Option<&mut T> {
		self.slots.iter_mut().rev().find_map(|slot| {
			if let VxSlot::Using { data, .. } = slot {
				Some(data)
			} else {
				None
			}
		})
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.len == 0
	}

	pub fn contains(&self, id: VxGenIndex) -> bool {
		match self.slots.get(id.index) {
			Some(VxSlot::Using { generation, .. }) => *generation == id.generation,
			_ => false,
		}
	}

	#[inline]
	pub fn len(&self) -> usize { self.len }
	#[inline]
	pub fn available_length(&self) -> usize { self.slots.len() }
	#[inline]
	pub fn free_slot_count(&self) -> usize { self.available_length() - self.len() }
	#[inline]
	pub fn capacity(&self) -> usize { self.slots.capacity() }

	//Iterators
	pub fn iter(&self) -> GenIter<'_, T> {
		GenIter { inner: self.slots.iter() }
	}
	pub fn iter_mut(&mut self) -> GenIterMut<'_, T> {
		GenIterMut { inner: self.slots.iter_mut() }
	}
	pub fn iter_with_id(&self) -> GenIterWithId<'_, T> {
		GenIterWithId { inner: self.slots.iter().enumerate() }
	}
	pub fn iter_with_id_mut(&mut self) -> GenIterWithIdMut<'_, T> {
		GenIterWithIdMut { inner: self.slots.iter_mut().enumerate() }
	}

	#[inline]
	pub fn clear(&mut self) {
		self.slots.clear();
		self.free_head = None;
		self.len = 0;
	}
}

#[macro_export]
macro_rules! vx_gen_vector {
	() => {
		VxGenVector::new()
	};
}