-- A simple scene with some miscellaneous geometry.

mat1 = gr.cook_torrance({0.72, 0.45, 0.2}, {0.72, 0.45, 0.2}, 0.0, 0.3, 0.863, 2.639)

scene_root = gr.node('root')

s1 = gr.nh_sphere('s1', {0, 0, -200}, 300)
scene_root:add_child(s1)
s1:set_material(mat1)

white_light = gr.light({400, 1000, 1000}, {1.0, 1.0, 1.0}, {1, 0, 0})

gr.render(scene_root, 'cook-torrance-test.png', 1024, 1024,
	  {0, 0, 800}, {0, 0, -1}, {0, 1, 0}, 50,
	  {0.05, 0.05, 0.05}, {white_light})
