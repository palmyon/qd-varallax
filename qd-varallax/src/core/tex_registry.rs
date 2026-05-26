use std::{collections::HashMap, };

use crate::types::genelational_vector::VxGenIndex;


type VxTextureLabelHash = u64;

pub(crate) struct VxTextureRegistry {
	cache: HashMap<VxTextureLabelHash, VxGenIndex>
}


impl VxTextureRegistry {
	pub fn new() -> Self {
		Self {
			cache: HashMap::new()
		}
	}

	pub fn get_slot(&self, id: VxTextureLabelHash) -> Option<VxGenIndex> {
		self.cache.get(&id).copied()
	}
	pub fn register(&mut self, id: VxTextureLabelHash, index: VxGenIndex) {
		self.cache.insert(id, index);
	}
	pub fn remove(&mut self, id: VxTextureLabelHash) {
		self.cache.remove(&id);
	}
	pub fn len(&self) -> u32 {
		self.cache.len() as u32
	}
}
