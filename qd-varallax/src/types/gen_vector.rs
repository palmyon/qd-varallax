//構造体
//Generation Index
#[derive(Hash, Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct VxGenIndex {
	pub(crate) index: usize,
	pub(crate) generation: u64,
}

//Slot
pub(crate) enum VxGenSlot<T> {
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
pub struct VxGenIterator<'a, T> {
	inner: std::slice::Iter<'a, VxGenSlot<T>>,
}

//Generation Iterator (mut)
pub struct VxGenIteratorMut<'a, T> {
	inner: std::slice::IterMut<'a, VxGenSlot<T>>,
}

//Generation Iterator and ID
pub struct VxGenIteratorWithId<'a, T> {
	inner: std::iter::Enumerate<std::slice::Iter<'a, VxGenSlot<T>>>,
}

//Generation Iterator and ID (mut)
pub struct VxgenIteratorWithIdMut<'a, T> {
	inner: std::iter::Enumerate<std::slice::IterMut<'a, VxGenSlot<T>>>,
}

//Generation Vector
pub struct VxGenVector<T> {
	pub(crate) slots: Vec<VxGenSlot<T>>,
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
impl <'a, T> Iterator for VxGenIterator<'a, T> {
	type Item = &'a T;
	
	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.inner.next()? {
				VxGenSlot::Using { data, .. } => return Some(data),
				VxGenSlot::Free { .. } => continue,
			}
		}
	}
}

//Generation Iterator (mut)
impl <'a, T> Iterator for VxGenIteratorMut<'a, T> {
	type Item = &'a mut T;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.inner.next()? {
				VxGenSlot::Using { data, .. } => return Some(data),
				VxGenSlot::Free { .. } => continue,
			}
		}
	}
}

//Generation Iterator and ID
impl <'a, T> Iterator for VxGenIteratorWithId<'a, T> {
	type Item = (VxGenIndex, &'a T);

	fn next(&mut self) -> Option<Self::Item> {
		while let Some((index, slot)) = self.inner.next() {
			if let VxGenSlot::Using { data, generation } = slot {
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
impl <'a, T> Iterator for VxgenIteratorWithIdMut<'a, T> {
	type Item = (VxGenIndex, &'a mut T);

	fn next(&mut self) -> Option<Self::Item> {
		while let Some((index, slot)) = self.inner.next() {
			if let VxGenSlot::Using { data, generation } = slot {
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
		self.get(index).expect("VxGenVector> Critical: Invalid Generation Index")
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

	//control
	pub fn insert(&mut self, data: T) -> VxGenIndex {
		self.insert_with_key(|_| data)
	}

	pub fn insert_with_key<F: FnOnce(VxGenIndex) -> T>(&mut self, f: F) -> VxGenIndex {
		if let Some(index) = self.free_head {
			match &mut self.slots[index] {
				VxGenSlot::Free { next_free, generation } => {
					let g = *generation;
					let id = VxGenIndex::new(index, g);

					let data = f(id);

					self.free_head = *next_free;
					self.slots[index] = VxGenSlot::Using { data, generation: g };
					self.len += 1;
					id
				}
				_ => unreachable!("free_head pointed to a Using slot."),
			}
		} else {
			let index = self.slots.len();
			let generation = 0;
			let id = VxGenIndex::new(index, generation);
			let data = f(id);
			self.slots.push(VxGenSlot::Using { data, generation });
			self.len += 1;
			id
		}
	}

	pub fn remove(&mut self, id: VxGenIndex) -> Option<T> {
		let slot = self.slots.get_mut(id.index)?;

		match slot {
			VxGenSlot::Using { generation , .. } if *generation == id.generation => {
				let old_gen= *generation;
				let old_slot = std::mem::replace(
					slot,
					VxGenSlot::Free {
						next_free: self.free_head,
						generation: old_gen + 1
					}
				);
				if let VxGenSlot::Using { data, .. } = old_slot {
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
			VxGenSlot::Using {
				data,
				generation
			} if *generation == id.generation => Some(data),
			_ => None,
		}
	}
	pub fn get_mut(&mut self, id: VxGenIndex) -> Option<&mut T> {
		match self.slots.get_mut(id.index)? {
			VxGenSlot::Using {
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
				VxGenSlot::Using { data: d_min, generation: g_min },
				VxGenSlot::Using { data: d_max, generation: g_max }
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
			if let VxGenSlot::Using { data, .. } = slot {
				Some(data)
			} else {
				None
			}
		})
	}

	pub fn last_mut(&mut self) -> Option<&mut T> {
		self.slots.iter_mut().rev().find_map(|slot| {
			if let VxGenSlot::Using { data, .. } = slot {
				Some(data)
			} else {
				None
			}
		})
	}

	pub fn last_id(&self) -> Option<VxGenIndex> {
		self.slots.iter()
			.enumerate()
			.rev()
			.find_map(|(index, slot)| {
				if let VxGenSlot::Using { generation, .. } = slot {
					Some(VxGenIndex::new(index, *generation))
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
			Some(VxGenSlot::Using { generation, .. }) => *generation == id.generation,
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
	pub fn iter(&self) -> VxGenIterator<'_, T> {
		VxGenIterator { inner: self.slots.iter() }
	}
	pub fn iter_mut(&mut self) -> VxGenIteratorMut<'_, T> {
		VxGenIteratorMut { inner: self.slots.iter_mut() }
	}
	pub fn iter_with_id(&self) -> VxGenIteratorWithId<'_, T> {
		VxGenIteratorWithId { inner: self.slots.iter().enumerate() }
	}
	pub fn iter_with_id_mut(&mut self) -> VxgenIteratorWithIdMut<'_, T> {
		VxgenIteratorWithIdMut { inner: self.slots.iter_mut().enumerate() }
	}
	#[inline]
	pub fn clear(&mut self) {
		self.slots.clear();
		self.free_head = None;
		self.len = 0;
	}
}