extends RigidBody2D

func _physics_process(_delta: float) -> void:
	self.apply_central_force(Vector2(randf_range(-1., 1.), randf_range(-1., 1.)).normalized() * 300.)
