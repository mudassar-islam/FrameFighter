extends Node2D

@onready var fighterInput = $FighterInput
@onready var historyLabel = $HUD/History
@onready var movesLabel = $HUD/Moves

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	movesLabel.text = ""
	fighterInput.set_side(FrameFighter.PLAYER_ONE)
	fighterInput.set_can_charge(true)

	pass # Replace with function body.

var accum = 0;

func _physics_process(_delta: float) -> void:
	accum += _delta

	# if accum >= 0.015:
	if accum >= 0.25:
		var actions = fighterInput.process_frame()
		var moves = fighterInput.matched_moves()
		for move in moves:
			movesLabel.text += move.name + " - " + str(move.perfect_input) + " - " + str(move.total_frames) + "\n"
		accum = 0

	var history = fighterInput.history()
	historyLabel.text = ""
	for item in history:
		historyLabel.text += str(item.frames) + " : " + item.movement + " - " + str(item.basic_actions) + " - " + str(item.composite_actions) + "\n"

	#print("\n\n")
	#print("Frames: " + str(actions.frames))
	#print("Movement: " + actions.movement)
	#print(actions.charge)

	pass
