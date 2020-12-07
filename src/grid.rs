use na::{Vector3, Vector4};
use util::math;
use Object;
use Ray;
use Hit;
use Material;

struct Grid {
    position: Vector3<f32>,
    size: Vector3<f32>,
    num_cells: Vector3<usize>,
    cell_size: f32,
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

                    grid.cells.push(GridCell::new(&position, grid_cell_size, &grid.objects));
                }
            }
        }

        grid
    }

    pub fn check_hit(&self, ray: &Ray) -> Option<(Hit, &Material)> {
        None
    }

    fn cell_at(&self, x: usize, y: usize, z: usize) -> &GridCell {
        &self.cells[x + (y * self.num_cells.x) + (z * self.num_cells.x * self.num_cells.y)]
    }
}

struct GridCell {
    objects: Vec<usize>,
}

impl GridCell {
    pub fn new(position: &Vector3<f32>, size: f32, objects: &Vec<Object>) -> Self {
        let mut cell = GridCell {
            objects: Vec::new(),
        };

        objects.iter().enumerate().for_each(|(i, object)| {
            let planes = get_grid_planes(position, size);
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

    pub fn check_hit(&self, ray: &Ray, objects: &Vec<Object>) -> Option<(Hit, &Material)> {
        None
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
            .all(|plane| -> bool { distance_from_plane(point, plane) < 0.0 })
    })
}

fn get_grid_planes(position: &Vector3<f32>, size: f32) -> [(Vector4<f32>, Vector4<f32>); 6] {
    let lower = position.insert_row(3, 1.0);
    let upper = position.add_scalar(size).insert_row(3, 1.0);

    [
        (lower, Vector4::new(1.0, 0.0, 0.0, 0.0)),
        (lower, Vector4::new(0.0, 1.0, 0.0, 0.0)),
        (lower, Vector4::new(0.0, 0.0, 1.0, 0.0)),
        (upper, Vector4::new(-1.0, 0.0, 0.0, 0.0)),
        (upper, Vector4::new(0.0, -1.0, 0.0, 0.0)),
        (upper, Vector4::new(0.0, 0.0, -1.0, 0.0)),
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

    let lower_world = obj.get_transform() * lower;
    let upper_world = obj.get_transform() * upper;

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

    let points = [
        obj.get_transform() * lower, // Left-Bottom-Back
        obj.get_transform() * Vector4::new(upper.x, lower.y, lower.z, 1.0), // Right-Bottom-Back
        obj.get_transform() * Vector4::new(upper.x, upper.y, lower.z, 1.0), // Right-Top-Back
        obj.get_transform() * Vector4::new(lower.x, upper.y, lower.z, 1.0), // Left-Top-Back
        obj.get_transform() * Vector4::new(lower.x, upper.y, upper.z, 1.0), // Left-Top-Front
        obj.get_transform() * Vector4::new(lower.x, lower.y, upper.z, 1.0), // Left-Bottom-Front
        obj.get_transform() * Vector4::new(upper.x, lower.y, upper.z, 1.0), // Right-Bottom-Front
        obj.get_transform() * upper, // Right-Top-Front
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
