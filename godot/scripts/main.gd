extends Node

@onready var placing_factory_scene: Resource = preload("res://scenes/placing_factory.tscn")
@onready var factory_scene: Resource = preload("res://scenes/factory.tscn")

var item_in_placement: Node2D = null
var placing_item = false
var mouse_position: Vector2

func _process(_delta: float) -> void:
	if placing_item:
		item_in_placement.position = mouse_position

func _input(event):
	if event is InputEventKey:
		if event.keycode == Key.KEY_ESCAPE || event.keycode == Key.KEY_Q:
			quit()
	elif event is InputEventMouseMotion:
		mouse_position = event.position
	elif event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			%Camera.zoom *= 0.95
		elif event.button_index == MOUSE_BUTTON_WHEEL_UP:
			%Camera.zoom *= 1.05
		elif event.button_index == MOUSE_BUTTON_LEFT:
			if placing_item:
				var position = item_in_placement.position
				%Buildings.remove_child(item_in_placement)
				item_in_placement.queue_free()
				var factory_node: Node2D = factory_scene.instantiate()
				factory_node.position = position
				%Buildings.add_child(factory_node)
				placing_item = false

func quit():
	get_tree().quit()

func _notification(what):
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		quit()
		
func _on_create_factory_button_pressed() -> void:
	if placing_item:
		return
	self.item_in_placement = placing_factory_scene.instantiate()
	%Buildings.add_child(self.item_in_placement)
	placing_item = true
