use std::sync::Arc;
use rlua::UserData;
use rlua::UserDataMethods;

use shading;

pub struct Material
{
	inner: Arc<dyn shading::Material>,
}

impl Material
{
	pub fn new<T: shading::Material + 'static>(material: T) -> Self
	{
		Material {
			inner: Arc::new(material),
		}
	}

    pub fn get_inner(&self) -> &Arc<dyn shading::Material>
    {
        &self.inner
    }
}

impl UserData for Material
{
	fn add_methods<'lua, U: UserDataMethods<'lua, Self>>(_methods: &mut U) {}
}
