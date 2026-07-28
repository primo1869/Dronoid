extends Node2D

class_name World

const CREATE_FACTORY_BUTTON_FORMAT = "Create Factory (%d)"

@onready var placing_factory_scene: Resource = preload("res://scenes/placing_factory.tscn")
@onready var factory_scene: Resource = preload("res://scenes/factory.tscn")
@onready var mineral_scene: Resource = preload("res://scenes/mineral.tscn")
@onready var mouse_position = get_global_mouse_position()

var item_in_placement: Node2D = null
var placing_item = false
var panning = false
var selected_factory: Node2D = null
var next_id = 0
var popup_shown = false
var starting_minerals = 100
var minerals = 0
var factory_cost = 70
var tween: Tween = null
var info_label_tween: Tween = null

var default_programs = [
	Program.new("Mining", ""),
	Program.new("Fighting", ""),
	Program.new("Exploring", ""),
	Program.new("Custom1", ""),
	Program.new("Custom2", ""),
]

func _ready() -> void:
	minerals_variation(starting_minerals)
	%CreateFactoryButton.text = self.CREATE_FACTORY_BUTTON_FORMAT % self.factory_cost
	for i in 25:
		var node_position = Vector2(randf_range(-500, 500), randf_range(-500, 500))
		for j in range(3, 8):
			node_position += Vector2(randf_range(-25, 25), randf_range(-25, 25))
			var mineral_node: Node2D = self.mineral_scene.instantiate()
			mineral_node.position = node_position
			%Buildings.add_child(mineral_node)

func _process(_delta: float) -> void:
	if self.selected_factory:
		refresh_right_menu_buttons()
	if placing_item:
		self.item_in_placement.position = mouse_position
	var new_mouse_position = get_global_mouse_position()
	if panning:
		%Camera.position += (new_mouse_position - mouse_position) / 3
	mouse_position = new_mouse_position

func select_factory(new_selection: Node2D):
	if self.selected_factory:
		self.selected_factory.unselect()
	self.selected_factory = new_selection
	self.selected_factory.select()
	%FactoryIdLabel.text = self.selected_factory.name
	%ProgramSelectButton.text = self.selected_factory.program.name

func refresh_right_menu_buttons():
	%RefreshCooldownBar.value = self.selected_factory.timer / self.selected_factory.time_to_spawn * 100
	%AutoSpawnCheckButton.button_pressed = self.selected_factory.auto_spawn
	if self.selected_factory.cooldown:
		%SpawnDroneButton.disabled = true
	if self.selected_factory.auto_spawn:
		%SpawnDroneButton.disabled = true
	if !self.selected_factory.cooldown && !self.selected_factory.auto_spawn && self.selected_factory.timer == 0.:
		%SpawnDroneButton.disabled = false
		%AutoSpawnCheckButton.disabled = false
	if self.selected_factory:
		%ProgramSelectButton.disabled = false

func _input(event):
	if event is InputEventKey:
		if event.keycode == Key.KEY_ESCAPE || event.keycode == Key.KEY_Q:
			quit()
	elif event is InputEventMouseButton:
		if self.popup_shown:
			return
		if event.is_pressed():
			if event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
				if %Camera.zoom.x > 0.05:
					%Camera.zoom -= Vector2(0.05, 0.05)
					%Camera.zoom = %Camera.zoom.clamp(Vector2(0, 0), Vector2(2, 2))
			elif event.button_index == MOUSE_BUTTON_WHEEL_UP:
				%Camera.zoom += Vector2(0.05, 0.05)
				%Camera.zoom = %Camera.zoom.clamp(Vector2(0, 0), Vector2(2, 2))
			elif event.button_index == MOUSE_BUTTON_LEFT:
				if placing_item:
					var item_position = item_in_placement.position
					%Buildings.remove_child(item_in_placement)
					self.item_in_placement.queue_free()
					var factory_node: Node2D = factory_scene.instantiate()
					factory_node.position = item_position
					factory_node.name = "Factory %s" % str(next_id)
					next_id += 1
					%Buildings.add_child(factory_node)
					self.placing_item = false
			elif event.button_index == MOUSE_BUTTON_MIDDLE:
				self.panning = true
		if event.is_released():
			if event.button_index == MOUSE_BUTTON_MIDDLE:
				self.panning = false

func quit():
	get_tree().quit()

func _notification(what):
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		quit()

func set_message_label(text: String):
	%MessageLabel.modulate.a = 1.
	%MessageLabel.text = text
	if self.tween:
		self.tween.kill()
	self.tween = create_tween()	
	self.tween.tween_property(%MessageLabel, "modulate:a", 0, 5.).set_trans(Tween.TRANS_EXPO)

func hide_info_label():
	if self.info_label_tween:
		self.info_label_tween.kill()
	self.info_label_tween = create_tween()
	self.info_label_tween.tween_property(%InfoLabel, "modulate:a", 0, 2.).set_trans(Tween.TRANS_EXPO)

func set_info_label(text: String):
	if self.info_label_tween:
		self.info_label_tween.kill()
	%InfoLabel.modulate.a = 1
	%InfoLabel.text = text

func minerals_variation(delta):
	self.minerals += delta
	%MineralCountLabel.text = str(self.minerals)

func _on_create_factory_pressed() -> void:
	if self.placing_item:
		return
	if minerals < self.factory_cost:
		set_message_label("Inssuficient minerals")
		return
	minerals_variation(-self.factory_cost)
	self.item_in_placement = self.placing_factory_scene.instantiate()
	%Buildings.add_child(self.item_in_placement)
	self.placing_item = true

func _on_spawn_drone_button_pressed() -> void:
	self.selected_factory.toggle_spawn_drone()

func _on_auto_spawn_check_button_toggled(toggled_on: bool) -> void:
	self.selected_factory.set_auto_spawn(toggled_on)

func hide_program_pick_popup():
	%ProgramSelectButton.text = self.selected_factory.program.name
	%ProgramPickPopup.hide()
	self.popup_shown = false

func show_program_pick_popup():
	%ProgramSelectButton.text = "Close"
	%ProgramList.clear()
	for program in default_programs:
		%ProgramList.add_item(program.name)
	%ProgramPickPopup.show()
	self.popup_shown = true

func _on_program_select_button_pressed() -> void:
	if !%ProgramPickPopup.visible:
		show_program_pick_popup()
	else:
		hide_program_pick_popup()

func _on_program_list_item_activated(index: int) -> void:
	self.selected_factory.program.name = %ProgramList.get_item_text(index)
	hide_program_pick_popup()
