extends Node2D

var integrity = 100

@onready var world_ref: World = get_tree().get_nodes_in_group("world").front()

func _on_mouse_entered() -> void:
	get_node_or_null("Normal").hide()
	get_node_or_null("Hovered").show()
	self.world_ref.set_info_label(name)

func _on_mouse_exited() -> void:
	get_node_or_null("Normal").show()
	get_node_or_null("Hovered").hide()
	self.world_ref.hide_info_label()
