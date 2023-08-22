
scene_root = gr.node('root')

white_wall = gr.lambertian({1.0, 1.0, 1.0})
red_wall = gr.lambertian({1.0, 0.0, 0.0})
green_wall = gr.lambertian({0.0, 1.0, 0.0})

left_wall = gr.cube('left_wall')
scene_root:add_child(left_wall)
left_wall:scale(0.2, 5.0, 5.0)
left_wall:translate(-(2.5 + 0.2), 0.0, -5.0)
left_wall:set_material(red_wall)

right_wall = gr.cube('right_wall')
scene_root:add_child(right_wall)
right_wall:scale(0.2, 5.0, 5.0)
right_wall:translate(2.5, 0.0, -5.0)
right_wall:set_material(green_wall)

floor = gr.cube('floor')
scene_root:add_child(floor)
floor:scale(5.0 + (0.2 * 2), 0.2, 5.0 + 0.2)
floor:translate(-(2.5 + 0.2), -0.2, -(5.0 + 0.2))
floor:set_material(white_wall)

back_wall = gr.cube('back_wall')
scene_root:add_child(back_wall)
back_wall:scale(5.0 + (0.2 * 2.0), 5.0, 0.2)
back_wall:translate(-(2.5 + 0.2), 0.0, -(5.0 + 0.2))
back_wall:set_material(white_wall)

ceiling = gr.cube('ceiling')
scene_root:add_child(ceiling)
ceiling:scale(5.0 + (0.2 * 2.0), 0.2, 5.0 + 0.2)
ceiling:translate(-(2.5 + 0.2), 5.0, -(5.0 + 0.2))
ceiling:set_material(white_wall)

left_floor_box = gr.cube('left_floor_box')
scene_root:add_child(left_floor_box)
left_floor_box:scale(1.5, 3.0, 1.5)
left_floor_box:rotate('Y', 20)
left_floor_box:translate(-1.75, 0.0, -4.0)
left_floor_box:set_material(white_wall)

right_floor_box = gr.cube('right_floor_box')
scene_root:add_child(right_floor_box)
right_floor_box:scale(1.5, 1.5, 1.5)
right_floor_box:rotate('Y', -20)
right_floor_box:translate(0.2, 0.0, -3.0)
right_floor_box:set_material(white_wall)

white_light = gr.light({0, 4.5, -2.5}, {1.0, 1.0, 1.0}, {1, 0, 0})

gr.render(scene_root, 'cornell-box-1.png', 1024, 1024,
	{0, 2.5, 5.4}, {0, 2.5, 0}, {0, 1, 0}, 50,
	{0, 0, 0}, {white_light})
