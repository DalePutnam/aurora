-- A simple scene with five spheres

mat1 = gr.material({0.4, 0.7, 0.4}, {0.1, 0.1, 0.1}, 5)
hide = gr.material({0.84, 0.6, 0.53}, {0.3, 0.3, 0.3}, 20)
whide = gr.material({0.9, 0.9, 0.9}, {0.3, 0.3, 0.3}, 20)

cow_poly = gr.mesh('cow', 'test/assets/cow.obj')
factor = 2.0/(1.1+2.637)
cow_poly:set_material(hide)
cow_poly:translate(0.0, 3.637, 0.0)
cow_poly:scale(factor, factor, factor)
cow_poly:translate(0.0, -1.0, 0.0)

wcow_poly = gr.mesh('cow2', 'test/assets/cow.obj')
wfactor = 2.0/(1.1+2.637)
wcow_poly:set_material(whide)
wcow_poly:translate(0.0, 3.637, 0.0)
wcow_poly:scale(factor, factor, factor)
wcow_poly:translate(0.0, -1.0, 0.0)

scene_root = gr.node('root')
scene_root:translate(0, 0, 780)

planet_one = gr.node('planet1')
scene_root:add_child(planet_one)
planet_one:translate(-6, -12, 0)

sp1 = gr.sphere('sp1')
planet_one:add_child(sp1)
sp1:scale(10, 10, 10)
sp1:set_material(mat1)

cow1 = gr.node('cow1')
planet_one:add_child(cow1)
cow1:add_child(cow_poly)
cow1:translate(0, 10.8, 0)
cow1:rotate('y', 55)

planet_two = gr.node('planet2')
scene_root:add_child(planet_two)
planet_two:translate(20, 5, -100)

sp2 = gr.sphere('sp2')
planet_two:add_child(sp2)
sp2:scale(10, 10, 10)
sp2:set_material(mat1)

cow2 = gr.node('cow2')
planet_two:add_child(cow2)
cow2:add_child(wcow_poly)
cow2:translate(0, 7.6, 0)
cow2:rotate('y', -145)
cow2:scale(1.5, 1.5, 1.5)

white_light = gr.light({-6.0, 100.0, 780.0}, {0.9, 0.9, 0.9}, {1, 0, 0})
orange_light = gr.light({400.0, 100.0, 150.0}, {0.7, 0.0, 0.7}, {1, 0, 0})

gr.render(scene_root, 'simple.png', 1024, 1024,
	  {0, 0, 800}, {0, 0, -800}, {0, 1, 0}, 50,
	  {0.3, 0.3, 0.3}, {white_light})--, orange_light})
