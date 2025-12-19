use godot::prelude::*;

mod internal;
mod addons;

struct FrameFighter;

#[gdextension]
unsafe impl ExtensionLibrary for FrameFighter {}
