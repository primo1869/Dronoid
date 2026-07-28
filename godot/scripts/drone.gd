extends RigidBody2D

class_name Drone

var program: Program = null

@onready var world_ref: World = get_tree().get_nodes_in_group("world").front()

func _physics_process(_delta: float) -> void:
	self.apply_central_force(Vector2(randf_range(-1., 1.), randf_range(-1., 1.)).normalized() * 300.)
	
func _on_mouse_entered() -> void:
	get_node_or_null("Normal").hide()
	get_node_or_null("Hovered").show()
	self.world_ref.set_info_label("Drone")

func _on_mouse_exited() -> void:
	get_node_or_null("Normal").show()
	get_node_or_null("Hovered").hide()
	self.world_ref.hide_info_label()
