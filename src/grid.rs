use na::{Vector3, Vector4};
use BoundingBox;
use Object;

struct Grid {
    position: Vector3<f32>,
    size: Vector3<f32>,
    num_cells: Vector3<usize>,
    cells: Vec<GridCell>,
    objects: Vec<Object>,
}

impl Grid {
    fn new(objects: Vec<Object>) -> Self {
        let mut grid_min = Vector3::repeat(f32::INFINITY);
        let mut grid_max = Vector3::repeat(f32::NEG_INFINITY);

        let mut volume_sum = 0.0;

        for object in &objects {
            let inv_trans = object.get_transform().try_inverse().unwrap();
            let (min, max) = object.get_bounding_box().get_extents();

            let mut corners = Vec::new();

            corners.push(inv_trans * Vector4::new(min.x, min.y, min.z, 1.0));
            corners.push(inv_trans * Vector4::new(min.x, min.y, max.z, 1.0));
            corners.push(inv_trans * Vector4::new(min.x, max.y, min.z, 1.0));
            corners.push(inv_trans * Vector4::new(min.x, max.y, max.z, 1.0));
            corners.push(inv_trans * Vector4::new(max.x, min.y, min.z, 1.0));
            corners.push(inv_trans * Vector4::new(max.x, min.y, max.z, 1.0));
            corners.push(inv_trans * Vector4::new(max.x, max.y, min.z, 1.0));
            corners.push(inv_trans * Vector4::new(max.x, max.y, max.z, 1.0));

            for corner in &corners {
                grid_min.x = f32::min(grid_min.x, corner.x);
                grid_min.y = f32::min(grid_min.y, corner.y);
                grid_min.z = f32::min(grid_min.z, corner.z);

                grid_max.x = f32::max(grid_max.x, corner.x);
                grid_max.y = f32::max(grid_max.y, corner.y);
                grid_max.z = f32::max(grid_max.z, corner.z);
            }

            let vec_x = corners[4] - corners[0];
            let vec_y = corners[1] - corners[0];
            let vec_z = corners[2] - corners[0];

            let size_x = f32::abs(vec_x.dot(&vec_x));
            let size_y = f32::abs(vec_y.dot(&vec_y));
            let size_z = f32::abs(vec_z.dot(&vec_z));

            volume_sum += size_x * size_y * size_z;
        }

        let average_volume = volume_sum / objects.len() as f32;
        let grid_cell_size = f32::powf(average_volume / 8.0, 1.0 / 3.0);

        let num_cells = Vector3::new(
            f32::ceil((grid_max.x - grid_min.x) / grid_cell_size) as usize,
            f32::ceil((grid_max.y - grid_min.y) / grid_cell_size) as usize,
            f32::ceil((grid_max.z - grid_min.z) / grid_cell_size) as usize,
        );

        let grid_size = Vector3::new(
            num_cells.x as f32 * grid_cell_size,
            num_cells.y as f32 * grid_cell_size,
            num_cells.z as f32 * grid_cell_size,
        );

        let mut cells = vec![GridCell::new(); num_cells.x * num_cells.y * num_cells.z];

        for z in 0..num_cells.z {
            for y in 0..num_cells.y {
                for x in 0..num_cells.x {
                    let position = grid_min
                        + Vector3::new(
                            grid_cell_size * x as f32,
                            grid_cell_size * y as f32,
                            grid_cell_size * z as f32,
                        );

                    cells[x + (y * num_cells.x) + (z * num_cells.x * num_cells.y)].add_objects(
                        &position,
                        grid_cell_size,
                        &objects,
                    )
                }
            }
        }

        Grid {
            position: grid_min,
            size: grid_size,
            num_cells: num_cells,
            cells: cells,
            objects: objects,
        }
    }
}

#[derive(Clone)]
struct GridCell {
    objects: Vec<usize>,
}

impl GridCell {
    pub fn new() -> Self {
        GridCell {
            objects: Vec::new(),
        }
    }

    pub fn add_objects(&mut self, position: &Vector3<f32>, size: f32, objects: &Vec<Object>) {
        let planes = get_grid_planes(position, size);

        objects.iter().enumerate().for_each(|(i, object)| {
            let polygons = get_bbox_polygons(object);

            if check_polygons_in_cell(&planes, polygons) {
                self.objects.push(i);
            }
        });
    }
}

fn check_polygons_in_cell(
    planes: &Vec<(Vector4<f32>, Vector4<f32>)>,
    polygons: Vec<Vec<Vector4<f32>>>,
) -> bool {
    for polygon in polygons {
        let mut output_list = polygon;

        for plane in planes {
            let input_list = output_list;
            output_list = Vec::new();

            for i in 0..input_list.len() {
                let current_point = input_list[i];
                let prev_point = input_list[(i + input_list.len() - 1) % input_list.len()];

                let la = (current_point - plane.0).dot(&plane.1);
                let lb = (prev_point - plane.0).dot(&plane.1);

                if la >= 0.0 {
                    if lb < 0.0 {
                        let t = la / (la - lb);
                        output_list.push(current_point + (t * (prev_point - current_point)));
                    }

                    output_list.push(current_point);
                } else if lb >= 0.0 {
                    let t = la / (la - lb);
                    output_list.push(current_point + (t * (prev_point - current_point)));
                }
            }
        }

        if !output_list.is_empty() {
            return true;
        }
    }

    false
}

fn get_grid_planes(position: &Vector3<f32>, size: f32) -> Vec<(Vector4<f32>, Vector4<f32>)> {
    let mut grid_planes = Vec::new();

    let lower = position.insert_row(3, 1.0);
    let upper = position.add_scalar(size).insert_row(3, 1.0);

    grid_planes.push((lower, Vector4::new(1.0, 0.0, 0.0, 0.0)));
    grid_planes.push((lower, Vector4::new(0.0, 1.0, 0.0, 0.0)));
    grid_planes.push((lower, Vector4::new(0.0, 0.0, 1.0, 0.0)));
    grid_planes.push((upper, Vector4::new(-1.0, 0.0, 0.0, 0.0)));
    grid_planes.push((upper, Vector4::new(0.0, -1.0, 0.0, 0.0)));
    grid_planes.push((upper, Vector4::new(0.0, 0.0, -1.0, 0.0)));

    grid_planes
}

fn get_bbox_polygons(obj: &Object) -> Vec<Vec<Vector4<f32>>> {
    let (lower, upper) = obj.get_bounding_box().get_extents();
    let mut polygons = Vec::new();

    polygons.push(vec![
        obj.get_transform() * lower,
        obj.get_transform() * Vector4::new(lower.x, upper.y, lower.z, 1.0),
        obj.get_transform() * Vector4::new(lower.x, upper.y, upper.z, 1.0),
        obj.get_transform() * Vector4::new(lower.x, upper.y, lower.z, 1.0),
    ]);

    polygons.push(vec![
        obj.get_transform() * lower,
        obj.get_transform() * Vector4::new(upper.x, lower.y, lower.z, 1.0),
        obj.get_transform() * Vector4::new(upper.x, lower.y, upper.z, 1.0),
        obj.get_transform() * Vector4::new(lower.x, lower.y, upper.z, 1.0),
    ]);

    polygons.push(vec![
        obj.get_transform() * lower,
        obj.get_transform() * Vector4::new(upper.x, lower.y, lower.z, 1.0),
        obj.get_transform() * Vector4::new(upper.x, upper.y, lower.z, 1.0),
        obj.get_transform() * Vector4::new(lower.x, upper.y, lower.z, 1.0),
    ]);

    polygons.push(vec![
        obj.get_transform() * upper,
        obj.get_transform() * Vector4::new(upper.x, lower.y, upper.z, 1.0),
        obj.get_transform() * Vector4::new(upper.x, lower.y, lower.z, 1.0),
        obj.get_transform() * Vector4::new(upper.x, upper.y, lower.z, 1.0),
    ]);

    polygons.push(vec![
        obj.get_transform() * upper,
        obj.get_transform() * Vector4::new(lower.x, upper.y, upper.z, 1.0),
        obj.get_transform() * Vector4::new(lower.x, upper.y, lower.z, 1.0),
        obj.get_transform() * Vector4::new(upper.x, upper.y, lower.z, 1.0),
    ]);

    polygons.push(vec![
        obj.get_transform() * upper,
        obj.get_transform() * Vector4::new(upper.x, lower.y, upper.z, 1.0),
        obj.get_transform() * Vector4::new(lower.x, lower.y, upper.z, 1.0),
        obj.get_transform() * Vector4::new(lower.x, upper.y, upper.z, 1.0),
    ]);

    polygons
}
