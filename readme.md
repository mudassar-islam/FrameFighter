![icon](project/assets/icon.png)

# FrameFighter

An input processor for fighting games in the [Godot Game Engine](https://godotengine.org/). Made in Rust using GDExtension and [godot-rust](https://godot-rust.github.io/).

> [!IMPORTANT]
> This addon deals only with input processing needs for a fighting game. This is intended to be used within a larger project to handle user-input only. For a complete fighting-games toolkit, check out [Castagne Engine](https://castagneengine.com/).

## Feature Status

- [x] Input processing with user-defined Actions & Move Lists.
- [x] Basic Actions i.e. Buttons on an arcade stick.
- [x] Composite Actions i.e. hold 2 punches for an EX Punch or Parry.
- [x] Movement with side switching & SOCD cleaning.
- [x] Input history tracking. 
- [x] Sequence Matching for moves.
- - [x] Per move modifiers for allowing/preventing leniency.
- - [ ] Charge Moves
- - [ ] Priority based matching.
- [ ] Custom in-engine GUI for customizing ActionMap & MoveList resources.

## Usage Example

1. Add a **FighterInput** node to your scene.
2. Create & add a **FighterActionMap** & **FighterMoveList** resource to the node.
3. Use it within your script:

```gdscript
@onready var fighterInput = $FighterInput


func _ready() -> void:
	fighterInput.set_side(FrameFighter.PLAYER_ONE)

func _physics_process(_delta) -> void:
	if is_on_floor():
		fighterInput.set_can_charge(true)
		
	fighterInput.set_side(get_player_side())
	
	var result = fighterInput.process_frame()
	
	for move in result.get_matched_moves():
		print(move.name)          # User-defined move name.
```
