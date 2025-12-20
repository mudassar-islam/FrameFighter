![icon](project/assets/icon.png)

# FrameFighter

An input processor for fighting games in the [Godot Game Engine](https://godotengine.org/) that handles input buffers, input histories, sequence matching with per-move customization, side switching & charge moves. Made in Rust using GDExtension and [godot-rust](https://godot-rust.github.io/).

## Feature Status

- [x] Input processing based on a user-defined ActionMap & MoveList
- [x] Basic Actions i.e. buttons on an arcade stick.
- [x] Composite Actions i.e. hold 2 punches for an EX Punch.
- [x] Side Switching
- [x] Charge Moves
- [ ] Sequence Matching for moves.
- - [ ] Per move modifiers for allowing/preventing leniency.
- - [ ] Priority system for conflicting matched moves.
- [ ] Special in-engine GUI for customizing ActionMap & MoveList resources.

## Usage
```gdscript
@onready var fighterInput = $FighterInput

func _physics_process(_delta) -> void:
	fighterInput.tick()
```
