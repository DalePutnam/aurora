use na::{Vector3, Vector4, U3, Matrix4};
use util::math;
use Hit;
use Material;
use Object;
use Ray;

pub struct Grid {
    position: Vector3<f32>,
    size: Vector3<f32>,
    num_cells: Vector3<usize>,
    cell_size: f32,
    cells: Vec<GridCell>,
    objects: Vec<Object>,
}

impl Grid {
    pub fn new(objects: Vec<Object>) -> Self {
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

            let size_x = f32::sqrt(f32::abs(vec_x.dot(&vec_x)));
            let size_y = f32::sqrt(f32::abs(vec_y.dot(&vec_y)));
            let size_z = f32::sqrt(f32::abs(vec_z.dot(&vec_z)));

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

        // let grid_size_scalar = f32::max(grid_max.x - grid_min.x, f32::max(grid_max.y - grid_min.y, grid_max.z - grid_min.z));

        // let num_cells = Vector3::new(200, 200, 200);
        // // let num_cells = Vector3::new(10, 10, 10);

        // let grid_size = Vector3::new(grid_size_scalar, grid_size_scalar, grid_size_scalar);
        // let grid_cell_size = grid_size_scalar / num_cells.x as f32;

        println!("Grid initialized at location {},{},{}", grid_min.x, grid_min.y, grid_min.z);
        println!("with side lengths {},{},{}", grid_size.x, grid_size.y, grid_size.z);
        println!("and cell size {}", grid_cell_size);
        println!("for {},{},{} cells in the x,y,z directions", num_cells.x, num_cells.y, num_cells.z);

        let mut grid = Grid {
            position: grid_min,
            size: grid_size,
            num_cells: num_cells,
            cell_size: grid_cell_size,
            cells: Vec::new(),
            objects: objects,
        };

        for z in 0..grid.num_cells.z {
            for y in 0..grid.num_cells.y {
                for x in 0..grid.num_cells.x {
                    let position = grid_min
                        + Vector3::new(
                            grid_cell_size * x as f32,
                            grid_cell_size * y as f32,
                            grid_cell_size * z as f32,
                        );

                    grid.cells
                        .push(GridCell::new(&position, grid_cell_size, &grid.objects, x, y, z));
                }
            }
        }

        grid
    }

    pub fn check_hit(&self, ray: &Ray) -> Option<(Hit, &Material)> {
        let ray_direction = ray.point - ray.origin;

        let step_x = if ray_direction.x >= 0.0 { 1 } else { -1 };
        let step_y = if ray_direction.y >= 0.0 { 1 } else { -1 };
        let step_z = if ray_direction.z >= 0.0 { 1 } else { -1 };

        let just_out_x = if ray_direction.x >= 0.0 {
            self.num_cells.x as i64
        } else {
            -1
        };
        let just_out_y = if ray_direction.y >= 0.0 {
            self.num_cells.y as i64
        } else {
            -1
        };
        let just_out_z = if ray_direction.z >= 0.0 {
            self.num_cells.z as i64
        } else {
            -1
        };

        let mut offset_x = ray.origin.x - self.position.x;
        let mut offset_y = ray.origin.y - self.position.y;
        let mut offset_z = ray.origin.z - self.position.z;

        let mut X = if offset_x <= 0.0 && offset_x >= -math::EPSILON {
            0
        } else if offset_x >= self.size.x && offset_x <= self.size.x + math::EPSILON {
            self.num_cells.x as i64 - 1
        } else {
            (offset_x / (self.size.x / self.num_cells.x as f32)) as i64
        };

        let mut Y = if offset_y <= 0.0 && offset_y >= -math::EPSILON {
            0
        } else if offset_y >= self.size.y && offset_y <= self.size.y + math::EPSILON {
            self.num_cells.y as i64 - 1
        } else {
            (offset_y / (self.size.y / self.num_cells.y as f32)) as i64
        };

        let mut Z = if offset_z <= 0.0 && offset_z >= -math::EPSILON {
            0
        } else if offset_z >= self.size.z && offset_z <= self.size.z + math::EPSILON {
            self.num_cells.z as i64 - 1
        } else {
            (offset_z / (self.size.z / self.num_cells.z as f32)) as i64
        };

        if X < 0
            || X >= self.num_cells.x as i64
            || Y < 0
            || Y >= self.num_cells.y as i64
            || Z < 0
            || Z >= self.num_cells.z as i64
        {
            if let Some(t) = self.intersect_grid_bounds(ray) {
                let grid_intersect = ray.origin + (t * (ray.point - ray.origin));

                //println!("Ray enters grid at {},{},{}", grid_intersect.x, grid_intersect.y, grid_intersect.z);

                offset_x = grid_intersect.x - self.position.x;
                offset_y = grid_intersect.y - self.position.y;
                offset_z = grid_intersect.z - self.position.z;

                X = if offset_x <= 0.0 && offset_x >= -math::EPSILON {
                    0
                } else if offset_x >= self.size.x && offset_x <= self.size.x + math::EPSILON {
                    self.num_cells.x as i64 - 1
                } else {
                    (offset_x / (self.size.x / self.num_cells.x as f32)) as i64
                };

                Y = if offset_y <= 0.0 && offset_y >= -math::EPSILON {
                    0
                } else if offset_y >= self.size.y && offset_y <= self.size.y + math::EPSILON {
                    self.num_cells.y as i64 - 1
                } else {
                    (offset_y / (self.size.y / self.num_cells.y as f32)) as i64
                };

                Z = if offset_z <= 0.0 && offset_z >= -math::EPSILON {
                    0
                } else if offset_z >= self.size.z && offset_z <= self.size.z + math::EPSILON {
                    self.num_cells.z as i64 - 1
                } else {
                    (offset_z / (self.size.z / self.num_cells.z as f32)) as i64
                };
            } else {
                return None;
            }
        }

        let mut cell = self.cell_at(X as usize, Y as usize, Z as usize);
        let cell_position = (Vector3::new(X as f32, Y as f32, Z as f32) * self.cell_size) + self.position;

        let o = ray.origin.fixed_rows::<U3>(0);
        let p = ray.point.fixed_rows::<U3>(0);

        // Initialize X values

        let first_point = cell_position
            + if step_x > 0 {
                Vector3::repeat(self.cell_size)
            } else {
                Vector3::repeat(0.0)
            };

        let second_point = cell_position
            + if step_x < 0 {
                Vector3::repeat(self.cell_size)
            } else {
                Vector3::repeat(0.0)
            };

        let normal = Vector3::new(1.0, 0.0, 0.0);

        let la = (o - first_point).dot(&normal);
        let lb = (p - first_point).dot(&normal);

        let mut t_max_x = f32::abs(la / (la - lb));

        let la = (o - second_point).dot(&normal);
        let lb = (p - second_point).dot(&normal);

        let t_delta_x = f32::abs(t_max_x - (la / (la - lb)));

        // Initialize Y values

        let first_point = cell_position
            + if step_y > 0 {
                Vector3::repeat(self.cell_size)
            } else {
                Vector3::repeat(0.0)
            };

        let second_point = cell_position
            + if step_y < 0 {
                Vector3::repeat(self.cell_size)
            } else {
                Vector3::repeat(0.0)
            };

        let normal = Vector3::new(0.0, 1.0, 0.0);

        let la = (o - first_point).dot(&normal);
        let lb = (p - first_point).dot(&normal);

        let mut t_max_y = f32::abs(la / (la - lb));

        let la = (o - second_point).dot(&normal);
        let lb = (p - second_point).dot(&normal);

        let t_delta_y = f32::abs(t_max_y - (la / (la - lb)));

        // Initialize Z values

        let first_point = cell_position
            + if step_z > 0 {
                Vector3::repeat(self.cell_size)
            } else {
                Vector3::repeat(0.0)
            };

        let second_point = cell_position
            + if step_z < 0 {
                Vector3::repeat(self.cell_size)
            } else {
                Vector3::repeat(0.0)
            };

        let normal = Vector3::new(0.0, 0.0, 1.0);

        let la = (o - first_point).dot(&normal);
        let lb = (p - first_point).dot(&normal);

        let mut t_max_z = f32::abs(la / (la - lb));

        let la = (o - second_point).dot(&normal);
        let lb = (p - second_point).dot(&normal);

        let t_delta_z = f32::abs(t_max_z - (la / (la - lb)));

        let mut found_hit = false;
        let mut hit = Hit {intersect: f32::INFINITY, normal: Vector4::repeat(0.0), uv: (0.0, 0.0) };
        let mut material = None;

        loop {
            //println!("Checking cell {},{},{}", X, Y, Z);

            if let Some(cell_hit) = cell.check_hit(ray, &self.objects) {
                if cell_hit.0.intersect < hit.intersect && cell_hit.0.intersect > math::EPSILON {
                    found_hit = true;
                    hit = cell_hit.0;
                    material = Some(cell_hit.1);
                }
            }

            if hit.intersect <= t_max_x && hit.intersect <= t_max_y && hit.intersect <= t_max_z {
                break;
            }

            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    X += step_x;
                    if X == just_out_x { break };

                    t_max_x += t_delta_x;
                } else {
                    Z += step_z;
                    if Z == just_out_z { break };

                    t_max_z += t_delta_z;
                }
            } else {
                if t_max_y < t_max_z {
                    Y += step_y;
                    if Y == just_out_y { break };

                    t_max_y += t_delta_y;
                } else {
                    Z += step_z;
                    if Z == just_out_z { break };

                    t_max_z += t_delta_z;
                }
            }

            cell = self.cell_at(X as usize, Y as usize, Z as usize);
        }

        if found_hit {
            Some((hit, material.unwrap()))
        } else {
            None
        }
    }

    fn intersect_grid_bounds(&self, ray: &Ray) -> Option<f32> {
        let ray_direction = ray.point - ray.origin;

        //println!("Ray Direction {},{},{}", ray_direction.x, ray_direction.y, ray_direction.z);

        let inv_direction = Vector4::repeat(1.0).component_div(&ray_direction);

        let min = (self.position.x - ray.origin.x) * inv_direction.x;
        let max = (self.position.x + self.size.x - ray.origin.x) * inv_direction.x;

        let (mut t_min, mut t_max) = if inv_direction.x >= 0.0 {
            (min, max)
        } else {
            (max, min)
        };

        let min = (self.position.y - ray.origin.y) * inv_direction.y;
        let max = (self.position.y + self.size.y - ray.origin.y) * inv_direction.y;

        let (ty_min, ty_max) = if inv_direction.y >= 0.0 {
            (min, max)
        } else {
            (max, min)
        };

        if (t_min > ty_max) || (ty_min > t_max) {
            return None;
        }

        if ty_min > t_min {
            t_min = ty_min;
        }

        if ty_max < t_max {
            t_max = ty_max;
        }

        let min = (self.position.z - ray.origin.z) * inv_direction.z;
        let max = (self.position.z + self.size.z - ray.origin.z) * inv_direction.z;

        let (tz_min, tz_max) = if inv_direction.z >= 0.0 {
            (min, max)
        } else {
            (max, min)
        };

        if (t_min > tz_max) || (tz_min > t_max) {
            return None;
        }

        if tz_min > t_min {
            t_min = tz_min;
        }

        if tz_max < t_max {
            t_max = tz_max;
        }

        if t_min > math::EPSILON {
            Some(t_min)
        } else if t_max > math::EPSILON {
            Some(t_max)
        } else {
            None
        }
    }

    fn cell_at(&self, x: usize, y: usize, z: usize) -> &GridCell {
        //println!("Accessing cell at {},{},{}", x, y, z);
        &self.cells[x + (y * self.num_cells.x) + (z * self.num_cells.x * self.num_cells.y)]
    }
}

struct GridCell {
    objects: Vec<usize>,
}

impl GridCell {
    pub fn new(position: &Vector3<f32>, size: f32, objects: &Vec<Object>, x: usize, y: usize, z: usize) -> Self {
        let mut cell = GridCell {
            objects: Vec::new(),
        };

        //println!("Populating cell {},{},{}, at position {},{},{} with size {}", x, y, z, position.x, position.y, position.z, size);

        objects.iter().enumerate().for_each(|(i, object)| {
            let planes = get_grid_planes(position, size, &object.get_transform());
            let polygons = get_bbox_polygons(object);

            if check_polygons_in_cell(&planes, &polygons) {
                cell.objects.push(i);
            } else {
                let bbox_planes = get_bbox_planes(object);
                let points = get_grid_points(position, size);

                if check_points_in_box(&bbox_planes, &points) {
                    cell.objects.push(i);
                }
            }
        });

        cell
    }

    pub fn check_hit<'a>(&self, ray: &Ray, objects: &'a Vec<Object>) -> Option<(Hit, &'a Material)> {
        let mut found_hit = false;
        let mut hit = Hit {intersect: f32::INFINITY, normal: Vector4::repeat(0.0), uv: (0.0, 0.0) };
        let mut material = None;

        for i in &self.objects {
            let obj = &objects[*i];

            if let Some(obj_hit) = obj.check_hit(ray) {
                if obj_hit.0.intersect < hit.intersect && obj_hit.0.intersect > math::EPSILON {
                    found_hit = true;
                    hit = obj_hit.0;
                    material = Some(obj_hit.1);
                }
            }
        }

        if found_hit {
            Some((hit, material.unwrap()))    
        } else {
            None
        }
    }
}

fn check_polygons_in_cell(
    planes: &[(Vector4<f32>, Vector4<f32>); 6],
    polygons: &[[Vector4<f32>; 4]; 6],
) -> bool {
    polygons.iter().any(|polygon| -> bool {
        let output_list = planes
            .iter()
            .fold(Vec::from(polygon.clone()), |input_list, plane| {
                clip_polygon_to_plane(&plane, input_list)
            });

        !output_list.is_empty()
    })
}

fn clip_polygon_to_plane(
    plane: &(Vector4<f32>, Vector4<f32>),
    input_list: Vec<Vector4<f32>>,
) -> Vec<Vector4<f32>> {
    // Sutherland-Hodgman algorithm

    (0..input_list.len()).fold(Vec::new(), |mut point_list, i| {
        let current_point = input_list[i];
        let prev_point = input_list[(i + input_list.len() - 1) % input_list.len()];

        let la = distance_from_plane(&current_point, plane);
        let lb = distance_from_plane(&prev_point, plane);

        if la >= 0.0 {
            if lb < 0.0 {
                point_list.push(intersection_from_distances(
                    la,
                    lb,
                    &current_point,
                    &prev_point,
                ));
            }

            point_list.push(current_point);
        } else if lb >= 0.0 {
            point_list.push(intersection_from_distances(
                la,
                lb,
                &current_point,
                &prev_point,
            ));
        }

        point_list
    })
}

fn distance_from_plane(point: &Vector4<f32>, plane: &(Vector4<f32>, Vector4<f32>)) -> f32 {
    (point - plane.0).dot(&plane.1)
}

fn intersection_from_distances(
    la: f32,
    lb: f32,
    current_point: &Vector4<f32>,
    prev_point: &Vector4<f32>,
) -> Vector4<f32> {
    let t = la / (la - lb);

    current_point + (t * (prev_point - current_point))
}

fn check_points_in_box(
    planes: &[(Vector4<f32>, Vector4<f32>); 6],
    points: &[Vector4<f32>; 8],
) -> bool {
    points.iter().any(|point| -> bool {
        planes
            .iter()
            .all(|plane| -> bool { distance_from_plane(point, plane) > 0.0 })
    })
}

fn get_grid_planes(position: &Vector3<f32>, size: f32, transform: &Matrix4<f32>) -> [(Vector4<f32>, Vector4<f32>); 6] {
    let lower = position.insert_row(3, 1.0);
    let upper = position.add_scalar(size).insert_row(3, 1.0);

    // [
    //     (lower, Vector4::new(1.0, 0.0, 0.0, 0.0)),
    //     (lower, Vector4::new(0.0, 1.0, 0.0, 0.0)),
    //     (lower, Vector4::new(0.0, 0.0, 1.0, 0.0)),
    //     (upper, Vector4::new(-1.0, 0.0, 0.0, 0.0)),
    //     (upper, Vector4::new(0.0, -1.0, 0.0, 0.0)),
    //     (upper, Vector4::new(0.0, 0.0, -1.0, 0.0)),
    // ]

    let inv_trans = transform.try_inverse().unwrap();

    let l = transform * lower;
    let u = transform * upper;

    [
        (l, math::local_to_world_normals(&Vector4::new(1.0, 0.0, 0.0, 0.0), &inv_trans)),
        (l, math::local_to_world_normals(&Vector4::new(0.0, 1.0, 0.0, 0.0), &inv_trans)),
        (l, math::local_to_world_normals(&Vector4::new(0.0, 0.0, 1.0, 0.0), &inv_trans)),
        (u, math::local_to_world_normals(&Vector4::new(-1.0, 0.0, 0.0, 0.0), &inv_trans)),
        (u, math::local_to_world_normals(&Vector4::new(0.0, -1.0, 0.0, 0.0), &inv_trans)),
        (u, math::local_to_world_normals(&Vector4::new(0.0, 0.0, -1.0, 0.0), &inv_trans)),
    ]
}

fn get_grid_points(position: &Vector3<f32>, size: f32) -> [Vector4<f32>; 8] {
    let lower = position.insert_row(3, 0.0);
    let upper = lower.add_scalar(size);

    [
        lower,                                        // Left-Bottom-Back
        Vector4::new(upper.x, lower.y, lower.z, 1.0), // Right-Bottom-Back
        Vector4::new(upper.x, upper.y, lower.z, 1.0), // Right-Top-Back
        Vector4::new(lower.x, upper.y, lower.z, 1.0), // Left-Top-Back
        Vector4::new(lower.x, upper.y, upper.z, 1.0), // Left-Top-Front
        Vector4::new(lower.x, lower.y, upper.z, 1.0), // Left-Bottom-Front
        Vector4::new(upper.x, lower.y, upper.z, 1.0), // Right-Bottom-Front
        upper,                                        // Right-Top-Front
    ]
}

fn get_bbox_planes(obj: &Object) -> [(Vector4<f32>, Vector4<f32>); 6] {
    let (lower, upper) = obj.get_bounding_box().get_extents();

    let inv_trans = obj.get_transform().try_inverse().unwrap();

    let lower_world = inv_trans * lower;
    let upper_world = inv_trans * upper;

    [
        (
            lower_world,
            math::local_to_world_normals(&Vector4::new(1.0, 0.0, 0.0, 0.0), obj.get_transform()),
        ),
        (
            lower_world,
            math::local_to_world_normals(&Vector4::new(0.0, 1.0, 0.0, 0.0), obj.get_transform()),
        ),
        (
            lower_world,
            math::local_to_world_normals(&Vector4::new(0.0, 0.0, 1.0, 0.0), obj.get_transform()),
        ),
        (
            upper_world,
            math::local_to_world_normals(&Vector4::new(-1.0, 0.0, 0.0, 0.0), obj.get_transform()),
        ),
        (
            upper_world,
            math::local_to_world_normals(&Vector4::new(0.0, -1.0, 0.0, 0.0), obj.get_transform()),
        ),
        (
            upper_world,
            math::local_to_world_normals(&Vector4::new(0.0, 0.0, -1.0, 0.0), obj.get_transform()),
        ),
    ]
}

fn get_bbox_polygons(obj: &Object) -> [[Vector4<f32>; 4]; 6] {
    let (lower, upper) = obj.get_bounding_box().get_extents();

    // let inv_trans = obj.get_transform().try_inverse().unwrap();

    let points = [
        /* inv_trans * */ *lower, // Left-Bottom-Back 0
        /* inv_trans * */ Vector4::new(upper.x, lower.y, lower.z, 1.0), // Right-Bottom-Back 1
        /* inv_trans * */ Vector4::new(upper.x, upper.y, lower.z, 1.0), // Right-Top-Back 2
        /* inv_trans * */ Vector4::new(lower.x, upper.y, lower.z, 1.0), // Left-Top-Back 3
        /* inv_trans * */ Vector4::new(lower.x, upper.y, upper.z, 1.0), // Left-Top-Front 4
        /* inv_trans * */ Vector4::new(lower.x, lower.y, upper.z, 1.0), // Left-Bottom-Front 5
        /* inv_trans * */ Vector4::new(upper.x, lower.y, upper.z, 1.0), // Right-Bottom-Front 6
        /* inv_trans * */ *upper, // Right-Top-Front 7
    ];

    [
        [points[0], points[3], points[4], points[5]], // Left
        [points[0], points[1], points[6], points[5]], // Bottom
        [points[0], points[1], points[2], points[3]], // Back
        [points[7], points[6], points[1], points[2]], // Right
        [points[7], points[4], points[3], points[2]], // Top
        [points[7], points[6], points[5], points[4]], // Front
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polygon_clip_fully_contained() {
        let planes = [
            (Vector4::new(-10.0, 0.0, 0.0, 1.0), Vector4::new(1.0, 0.0, 0.0, 1.0)),
            (Vector4::new(10., 0.0, 0.0, 1.0), Vector4::new(-1.0, 0.0, 0.0, 1.0)),
            (Vector4::new(0.0, -10.0, 0.0, 1.0), Vector4::new(0.0, 1.0, 0.0, 1.0)),
            (Vector4::new(0.0, 10.0, 0.0, 1.0), Vector4::new(0.0, -1.0, 0.0, 1.0)),
            (Vector4::new(0.0, 0.0, -10.0, 1.0), Vector4::new(0.0, 0.0, 1.0, 1.0)),
            (Vector4::new(0.0, 0.0, 10.0, 1.0), Vector4::new(0.0, 0.0, -1.0, 1.0))
        ];

        let points = [
            Vector4::new(-1.0, -1.0, -1.0, 1.0), // Left-Bottom-Back 0
            Vector4::new(1.0, -1.0, -1.0, 1.0), // Right-Bottom-Back 1
            Vector4::new(1.0, 1.0, -1.0, 1.0), // Right-Top-Back 2
            Vector4::new(-1.0, 1.0, -1.0, 1.0), // Left-Top-Back 3
            Vector4::new(-1.0, 1.0, 1.0, 1.0), // Left-Top-Front 4
            Vector4::new(-1.0, -1.0, 1.0, 1.0), // Left-Bottom-Front 5
            Vector4::new(1.0, -1.0, 1.0, 1.0), // Right-Bottom-Front 6
            Vector4::new(1.0, 1.0, 1.0, 1.0), // Right-Top-Front 7
        ];

        let polygons = [
            [points[0], points[3], points[4], points[5]], // Left
            [points[0], points[1], points[6], points[5]], // Bottom
            [points[0], points[1], points[2], points[3]], // Back
            [points[1], points[2], points[7], points[6]], // Right
            [points[2], points[3], points[4], points[7]], // Top
            [points[4], points[5], points[6], points[7]], // Front
        ];

        assert!(check_polygons_in_cell(&planes, &polygons));
        assert!(check_points_in_box(&planes, &points));
    }

    #[test]
    fn polygon_clip_fully_contained2() {
        let planes = [
            (Vector4::new(-1.0, 0.0, 0.0, 1.0), Vector4::new(1.0, 0.0, 0.0, 1.0)),
            (Vector4::new(1., 0.0, 0.0, 1.0), Vector4::new(-1.0, 0.0, 0.0, 1.0)),
            (Vector4::new(0.0, -1.0, 0.0, 1.0), Vector4::new(0.0, 1.0, 0.0, 1.0)),
            (Vector4::new(0.0, 1.0, 0.0, 1.0), Vector4::new(0.0, -1.0, 0.0, 1.0)),
            (Vector4::new(0.0, 0.0, -1.0, 1.0), Vector4::new(0.0, 0.0, 1.0, 1.0)),
            (Vector4::new(0.0, 0.0, 1.0, 1.0), Vector4::new(0.0, 0.0, -1.0, 1.0))
        ];

        let points = [
            Vector4::new(-10.0, -10.0, -10.0, 1.0), // Left-Bottom-Back 0
            Vector4::new(10.0, -10.0, -10.0, 1.0), // Right-Bottom-Back 1
            Vector4::new(10.0, 10.0, -10.0, 1.0), // Right-Top-Back 2
            Vector4::new(-10.0, 10.0, -10.0, 1.0), // Left-Top-Back 3
            Vector4::new(-10.0, 10.0, 10.0, 1.0), // Left-Top-Front 4
            Vector4::new(-10.0, -10.0, 10.0, 1.0), // Left-Bottom-Front 5
            Vector4::new(10.0, -10.0, 10.0, 1.0), // Right-Bottom-Front 6
            Vector4::new(10.0, 10.0, 10.0, 1.0), // Right-Top-Front 7
        ];

        let polygons = [
            [points[0], points[3], points[4], points[5]], // Left
            [points[0], points[1], points[6], points[5]], // Bottom
            [points[0], points[1], points[2], points[3]], // Back
            [points[1], points[2], points[7], points[6]], // Right
            [points[2], points[3], points[4], points[7]], // Top
            [points[4], points[5], points[6], points[7]], // Front
        ];

        assert!(!check_polygons_in_cell(&planes, &polygons));
    }
}