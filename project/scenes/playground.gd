extends Node2D

@onready var fighterInput = $FighterInput
		
# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	fighterInput.set_side(FrameFighter.PLAYER_ONE)
	fighterInput.should_charge(true)
	
	pass # Replace with function body.

# var accum = 0;

func _physics_process(_delta: float) -> void:
	fighterInput.process_frame()
	
	var actions = fighterInput.pressed_actions()
	
	print("Frames: ", str(actions.frames))
	print("Movement: " + actions.movement)
	print(actions.charge)
	
	print("Is Up Pressed: ", actions.is_pressed("up"))
	print("Down Charge: ", actions.get_charge_frames("down"))
	
	#print("\n\n")
	#print("Frames: " + str(actions.frames))
	#print("Movement: " + actions.movement)
	#print(actions.charge)
	
	pass
