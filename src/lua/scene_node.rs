use std::cell::Cell;
use std::clone::Clone;
use std::sync::Arc;
use std::sync::Mutex;

use lua;
use na::Matrix4;
use na::Unit;
use na::Vector3;
use primitives::Primitive;
use rlua;
use rlua::FromLua;
use rlua::UserData;
use rlua::UserDataMethods;
use rlua::Value;
use shading::Material;
use Object;

pub struct SceneNodeInner
{
	name: String,
	transform: Matrix4<f32>,
	children: Vec<SceneNode>,
	primitive: Option<Arc<dyn Primitive>>,
	material: Option<Arc<dyn Material>>,
	objects_built: Cell<usize>,
}

pub struct SceneNode
{
	inner: Arc<Mutex<SceneNodeInner>>,
}
impl SceneNode
{
	pub fn new(name: &str, primitive: Option<Arc<dyn Primitive>>) -> Self
	{
		let inner = SceneNodeInner {
			name: name.to_string(),
			transform: Matrix4::identity(),
			children: Vec::new(),
			primitive: primitive,
			material: None,
			objects_built: Cell::new(0),
		};

		SceneNode {
			inner: Arc::new(Mutex::new(inner)),
		}
	}

	pub fn convert_to_object_list(&self) -> Vec<Arc<Object>>
	{
		self.convert_to_object_list_private(Matrix4::identity())
	}

	pub fn set_material(&mut self, material: Arc<dyn Material>)
	{
		let mut node = self.inner.lock().unwrap();
		node.material = Some(material);
	}

	pub fn rotate(&mut self, axis: char, angle: f32)
	{
		let mut node = self.inner.lock().unwrap();

		let rotation_axis = match axis {
			'x' | 'X' => Vector3::new(1.0, 0.0, 0.0),
			'y' | 'Y' => Vector3::new(0.0, 1.0, 0.0),
			'z' | 'Z' => Vector3::new(0.0, 0.0, 1.0),
			_ => return,
		};

		let rotation_matrix =
			Matrix4::from_axis_angle(&Unit::new_normalize(rotation_axis), angle.to_radians());
		node.transform = rotation_matrix * node.transform;
	}

	pub fn scale(&mut self, amount: Vector3<f32>)
	{
		let mut node = self.inner.lock().unwrap();

		let scale_matrix = Matrix4::new_nonuniform_scaling(&amount);
		node.transform = scale_matrix * node.transform;
	}

	pub fn translate(&mut self, amount: Vector3<f32>)
	{
		let mut node = self.inner.lock().unwrap();

		let translate_matrix = Matrix4::new_translation(&amount);
		node.transform = translate_matrix * node.transform;
	}

	pub fn add_child(&mut self, child: SceneNode)
	{
		let mut node = self.inner.lock().unwrap();
		node.children.push(child);
	}

	fn convert_to_object_list_private(&self, transform: Matrix4<f32>) -> Vec<Arc<Object>>
	{
		let node = self.inner.lock().unwrap();

		let mut list = Vec::new();
		let cumulative_transform = transform * node.transform;

		if let Some(object) = self.build_object_with_transform(&node, cumulative_transform) {
			list.push(Arc::new(object));
		}

		for child in &node.children {
			let mut objects = child.convert_to_object_list_private(cumulative_transform);
			list.append(&mut objects);
		}

		list
	}

	fn build_object_with_transform(
		&self,
		node: &SceneNodeInner,
		transform: Matrix4<f32>,
	) -> Option<Object>
	{
		if let Some(primitive) = node.primitive.clone() {
			if let Some(material) = node.material.clone() {
				let object_number = node.objects_built.replace(node.objects_built.get() + 1);
				let object_name = format!("<{}>:{}", node.name, object_number);

				return Some(Object::new(object_name, transform, primitive, material));
			}
		}

		None
	}
}

impl Clone for SceneNode
{
	fn clone(&self) -> Self
	{
		SceneNode {
			inner: self.inner.clone(),
		}
	}
}

impl UserData for SceneNode
{
	fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T)
	{
		methods.add_method_mut("rotate", |lua, lua_node, (lua_axis, lua_angle)| {
			let axis = match lua_axis {
				Value::String(string) => match string.to_str() {
					Ok(slice) => slice.chars().nth(0).unwrap(),
					Err(e) => panic!("{}", e),
				},
				_ => panic!("Failed to rotate"),
			};

			let angle = f32::from_lua(lua_angle, lua)?;

			lua_node.rotate(axis, angle);

			Ok(())
		});

		methods.add_method_mut("scale", |lua, lua_node, (lua_x, lua_y, lua_z)| {
			let x = f32::from_lua(lua_x, lua)?;
			let y = f32::from_lua(lua_y, lua)?;
			let z = f32::from_lua(lua_z, lua)?;
			let amount = Vector3::new(x, y, z);

			lua_node.scale(amount);

			Ok(())
		});

		methods.add_method_mut("translate", |lua, lua_node, (lua_x, lua_y, lua_z)| {
			let x = f32::from_lua(lua_x, lua)?;
			let y = f32::from_lua(lua_y, lua)?;
			let z = f32::from_lua(lua_z, lua)?;
			let amount = Vector3::new(x, y, z);

			lua_node.translate(amount);

			Ok(())
		});

		methods.add_method_mut(
			"add_child",
			|_, lua_node, child_lua_node| match child_lua_node {
				Value::UserData(user_data) => match user_data.borrow::<SceneNode>() {
					Ok(child_node) => {
						lua_node.add_child(child_node.clone());
						Ok(())
					},
					Err(_) => panic!("Invalid node!"),
				},
				_ => panic!("Invalid node!"),
			},
		);

		methods.add_method_mut(
			"set_material",
			|_, lua_node, lua_material| match lua_material {
				Value::UserData(user_data) => match user_data.borrow::<lua::Material>() {
					Ok(material) => {
						lua_node.set_material(material.get_inner().clone());
						Ok(())
					},
					Err(e) => Err(e),
				},
				_ => Err(rlua::Error::RuntimeError(
					"set_material expected a LuaMaterial as its first argument".to_string(),
				)),
			},
		);
	}
}
