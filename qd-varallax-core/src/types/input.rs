use ahash::AHashMap;

use crate::types::geometry::VxVec2;



pub type VxKey = winit::keyboard::KeyCode;
pub type VxMouseButton = winit::event::MouseButton;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct VxMouseButtonState {
	pressed: bool,
	released: bool,
	// 押されているか(押し続けている含め)どうか
	down: bool,
}

impl VxMouseButtonState {
	#[inline]
	pub(crate) const fn new(pressed: bool, released: bool, down: bool) -> Self {
		Self { pressed, released, down }
	}
	#[inline]
	pub const fn is_pressed(&self) -> bool {
		self.pressed
	}
	#[inline]
	pub const fn is_released(&self) -> bool {
		self.released
	}
	#[inline]
	pub const fn is_down(&self) -> bool {
		self.down
	}
}


pub struct VxMouseState {
	pos: VxVec2,
	delta_vec: VxVec2,
	wheel_delta: VxVec2,
	buttons: AHashMap<VxMouseButton, VxMouseButtonState>,
}

impl VxMouseState {
	#[inline]
	pub fn new(pos: VxVec2, delta_vec: VxVec2, wheel_delta: VxVec2) -> Self {
		Self { pos, delta_vec, wheel_delta, buttons: AHashMap::new() }
	}
	#[inline]
	pub const fn pos(&self) -> VxVec2 {
		self.pos
	}
	#[inline]
	pub const fn delta_vec(&self) -> VxVec2 {
		self.delta_vec
	}
	#[inline]
	pub const fn wheel_delta(&self) -> VxVec2 {
		self.wheel_delta
	}
	#[inline]
	pub const fn buttons(&self) -> &AHashMap<VxMouseButton, VxMouseButtonState> {
		&self.buttons
	}
	#[inline]
	pub const fn buttons_mut(&mut self) -> &mut AHashMap<VxMouseButton, VxMouseButtonState> {
		&mut self.buttons
	}
}


#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct VxKeyState {
	pressed: bool,
	released: bool,
	down: bool,
	// Windowsならwindowsキー、MacならCommandキー
	super_key: bool,
}

impl VxKeyState {
	#[inline]
	pub(crate) const fn new(pressed: bool, released: bool, down: bool, super_key: bool) -> Self {
		Self { pressed, released, down, super_key }
	}
	#[inline]
	pub const fn is_pressed(&self) -> bool {
		self.pressed
	}
	#[inline]
	pub const fn is_released(&self) -> bool {
		self.released
	}
	#[inline]
	pub const fn is_down(&self) -> bool {
		self.down
	}
	#[inline]
	pub const fn is_super_key(&self) -> bool {
		self.super_key
	}
}


pub struct VxKeyboardState {
	keys: AHashMap<VxKey, VxKeyState>,
	text_input: String,
}

impl VxKeyboardState {
	#[inline]
	pub fn new(text_input: String) -> Self {
		Self { keys: AHashMap::new(), text_input }
	}
	#[inline]
	pub const fn keys(&self) -> &AHashMap<VxKey, VxKeyState> {
		&self.keys
	}
	#[inline]
	pub const fn keys_mut(&mut self) -> &mut AHashMap<VxKey, VxKeyState> {
		&mut self.keys
	}
	#[inline]
	pub const fn text_input(&self) -> &String {
		&self.text_input
	}
	#[inline]
	pub const fn text_input_mut(&mut self) -> &mut String {
		&mut self.text_input
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct VxKeyModifierState {
	ctrl: bool,
	shift: bool,
	alt: bool,
}

impl VxKeyModifierState {
	#[inline]
	pub(crate) const fn new(ctrl: bool, shift: bool, alt: bool) -> Self {
		Self { ctrl, shift, alt }
	}
	#[inline]
	pub const fn is_ctrl(&self) -> bool {
		self.ctrl
	}
	#[inline]
	pub const fn is_shift(&self) -> bool {
		self.shift
	}
	#[inline]
	pub const fn is_alt(&self) -> bool {
		self.alt
	}
}


pub struct VxInputState {
	mouse: VxMouseState,
	key: VxKeyboardState,
	modifier: VxKeyModifierState,
}

impl VxInputState {
	#[inline]
	pub(crate) const fn new(mouse: VxMouseState, key: VxKeyboardState, modifier: VxKeyModifierState) -> Self {
		Self { mouse, key, modifier }
	}
	#[inline]
	pub const fn mouse(&self) -> &VxMouseState {
		&self.mouse
	}
	#[inline]
	pub const fn mouse_mut(&mut self) -> &mut VxMouseState {
		&mut self.mouse
	}
	#[inline]
	pub const fn key(&self) -> &VxKeyboardState {
		&self.key
	}
	#[inline]
	pub const fn key_mut(&mut self) -> &mut VxKeyboardState {
		&mut self.key
	}
	#[inline]
	pub const fn modifier(&self) -> VxKeyModifierState {
		self.modifier
	}
	#[inline]
	pub const fn set_modifier_state(&mut self, modifier: VxKeyModifierState) {
		self.modifier = modifier;
	}
}