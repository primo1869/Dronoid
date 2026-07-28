extends Node2D

#var drone: Resource = preload("res://scenes/drone.tscn")
#
#func spawn_drone() -> void:
	#var new_drone: Node2D = drone.instantiate()
	#new_drone.position = position
	#get_parent().add_child(new_drone)

func _on_area_2d_mouse_entered() -> void:
	get_node_or_null("Sprite").hide()
	get_node_or_null("SpriteHovered").show()

func _on_area_2d_mouse_exited() -> void:
	get_node_or_null("Sprite").show()
	get_node_or_null("SpriteHovered").hide()
