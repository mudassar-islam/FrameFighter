extends Node2D

@onready var fighterInput = $FighterInput
		
# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	fighterInput.set_side(FrameFighter.PLAYER_ONE)
	fighterInput.should_charge(true)
	
	pass # Replace with function body.

var accum = 0;

func _physics_process(delta: float) -> void:
	fighterInput.tick()
	pass
