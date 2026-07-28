extends Node2D

var drone_scene: PackedScene = preload("res://scenes/drone.tscn")
var placed = false
var timer = 0.
var time_to_spawn = 5.
var auto_spawn = false
var cooldown = false

var program: Program = Program.new("Default", "return")

@onready var world_ref: World = get_tree().get_nodes_in_group("world").front()

func _process(delta: float) -> void:
	if cooldown || timer > 0.:
		timer += delta
		if timer > time_to_spawn:
			timer = 0
			if !auto_spawn:
				cooldown = false
			else:
				spawn_drone()

func toggle_spawn_drone():
	if self.cooldown:
		return
	spawn_drone()
	self.cooldown = true

func spawn_drone() -> void:
	var new_drone = self.drone_scene.instantiate() as Drone
	new_drone.position = self.position
	new_drone.program = self.program
	get_parent().add_child(new_drone)

func set_auto_spawn(value: bool):
	if !self.cooldown && value:
		spawn_drone()
	self.auto_spawn = value
	self.cooldown = value

func _on_mouse_entered() -> void:
	get_node_or_null("Normal").hide()
	get_node_or_null("Hovered").show()
	self.world_ref.set_info_label(name)

func _on_mouse_exited() -> void:
	get_node_or_null("Normal").show()
	get_node_or_null("Hovered").hide()
	self.world_ref.hide_info_label()

func select():
	get_node_or_null("Normal").hide()
	get_node_or_null("Selected").show()
	
func unselect():
	get_node_or_null("Normal").show()
	get_node_or_null("Selected").hide()

func _on_input_event(_viewport: Node, event: InputEvent, _shape_idx: int) -> void:
	if event is InputEventMouseButton:
		if event.is_pressed():
			if event.button_index == MOUSE_BUTTON_LEFT:
				if !placed:
					placed = true
					return
				get_parent().get_parent().select_factory(self)
