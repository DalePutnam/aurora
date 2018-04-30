use std::sync::Arc;
use core::Material;
use rlua::{UserData, UserDataMethods, Value};

pub struct LuaMaterial {
    material: Arc<Material>,
}

impl LuaMaterial {
    pub fn new(material: Material) -> Self {
        LuaMaterial {
            material: Arc::new(material),
        }
    }

    pub fn get_internal_material(&self) -> &Arc<Material> {
        &self.material
    }
}

impl UserData for LuaMaterial {
    fn add_methods(methods: &mut UserDataMethods<Self>) {

    }
}