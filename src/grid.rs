use std::cmp::Ordering;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

use na::Matrix4;
use na::Vector3;
use na::Vector4;
use num_cpus;
use rand;
use rand::seq::SliceRandom;
use shading::Material;
use util::math;
use Hit;
use Object;
use Ray;

pub struct Grid
{
	position: Vector3<f32>,
	size: Vector3<f32>,
	num_cells: Vector3<usize>,
	cell_size: f32,
	cells: Vec<GridCell>,
}

impl Grid
{
	pub fn new(objects: Vec<Arc<Object>>) -> Self
	{
		let bbox_corners = Grid::get_bbox_corners_in_world_space(&objects);

		let (grid_min, grid_max) = Grid::get_min_max_points(&bbox_corners);
		let average_volume = Grid::get_average_bbox_volume(&bbox_corners);

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

		// let num_cells_scalar = 200.0;
		// let grid_cell_size = f32::max(
		// 	grid_max.z - grid_min.z,
		// 	f32::max(grid_max.y - grid_min.y, grid_max.x - grid_min.x),
		// ) / num_cells_scalar;

		// let grid_size = Vector3::repeat(num_cells_scalar * grid_cell_size);
		// let num_cells = Vector3::repeat(num_cells_scalar as usize);

		println!(
			"Grid initialized at location {},{},{}",
			grid_min.x, grid_min.y, grid_min.z
		);
		println!(
			"with side lengths {},{},{}",
			grid_size.x, grid_size.y, grid_size.z
		);
		println!("and cell size {}", grid_cell_size);
		println!(
			"for {},{},{} cells in the x,y,z directions",
			num_cells.x, num_cells.y, num_cells.z
		);

		// Generate list of all grid cells to be populated
		let mut cell_list = Vec::new();
		for z in 0..num_cells.z {
			for y in 0..num_cells.y {
				for x in 0..num_cells.x {
					cell_list.push((x, y, z));
				}
			}
		}

		// Shuffle the list to more evenly distribute work between threads
		cell_list.shuffle(&mut rand::thread_rng());

		// We will need to share objects between threads so move it in an Arc
		let objects = Arc::new(objects);
		let cells_per_thread = cell_list.len() / num_cpus::get();

		let rx = {
			let (tx, rx) = mpsc::channel();

			for _ in 0..num_cpus::get() {
				// Split off a piece of the cell_list for the thread we are about to start
				let cell_list = cell_list.split_off(
					if cell_list.len() > cells_per_thread {
						cell_list.len() - cells_per_thread - 1
					} else {
						0
					},
				);

				let objects = Arc::clone(&objects);
				let tx = mpsc::Sender::clone(&tx);

				thread::spawn(move || {
					Grid::fill_worker(grid_min, grid_cell_size, cell_list, &objects, tx);
				});
			}

			rx
		};

		// Collect populated cells
		let mut cells: Vec<((usize, usize, usize), GridCell)> = rx.into_iter().collect();

		// Populated cells are in an arbitrary order, so we sort them by coordinates
		cells.sort_unstable_by(|cell1, cell2| -> Ordering {
			let ((x1, y1, z1), _) = cell1;
			let ((x2, y2, z2), _) = cell2;

			if z1 != z2 {
				usize::cmp(z1, z2)
			} else if y1 != y2 {
				usize::cmp(y1, y2)
			} else {
				usize::cmp(x1, x2)
			}
		});

		// Remove the x,y,z coordinates from the list of cells, they are no longer needed
		let cells = cells.into_iter().map(|(_, cell)| cell).collect();

		// Take objects out of the Arc, it no longer needs to be shared between threads
		//let objects = Arc::try_unwrap(objects).unwrap();

		Grid {
			position: grid_min,
			size: grid_size,
			num_cells: num_cells,
			cell_size: grid_cell_size,
			cells: cells,
		}
	}

	pub fn check_hit(&self, ray: &Ray) -> Option<(Hit, &dyn Material)>
	{
		let ray_direction = ray.point() - ray.origin();

		let (step_x, step_y, step_z) = self.get_step_directions(ray_direction);
		let (just_out_x, just_out_y, just_out_z) = self.get_step_out_values(ray_direction);

		let (mut grid_x, mut grid_y, mut grid_z) =
			if let Some(cell_coords) = self.get_starting_cell(ray) {
				cell_coords
			} else {
				return None;
			};

		let cell_position = (Vector4::new(grid_x as f32, grid_y as f32, grid_z as f32, 0.0)
			* self.cell_size)
			+ self.position.insert_row(3, 1.0);

		let (mut t_max_x, t_delta_x) =
			self.get_max_and_delta(step_x, ray, cell_position, Vector4::new(1.0, 0.0, 0.0, 0.0));

		let (mut t_max_y, t_delta_y) =
			self.get_max_and_delta(step_y, ray, cell_position, Vector4::new(0.0, 1.0, 0.0, 0.0));

		let (mut t_max_z, t_delta_z) =
			self.get_max_and_delta(step_z, ray, cell_position, Vector4::new(0.0, 0.0, 1.0, 0.0));

		let mut cell = self.cell_at(grid_x as usize, grid_y as usize, grid_z as usize);
		let mut hit: Option<(Hit, &dyn Material)> = None;

		loop {
			if let Some(cell_hit) = cell.check_hit(ray) {
				match &hit {
					Some((hit_info, _)) => {
						if cell_hit.0.intersect < hit_info.intersect
							&& math::far_from_zero_pos(cell_hit.0.intersect)
						{
							hit = Some(cell_hit);
						}
					},
					None => {
						hit = Some(cell_hit);
					},
				}
			}

			if let Some(hit) = &hit {
				if hit.0.intersect <= t_max_x
					&& hit.0.intersect <= t_max_y
					&& hit.0.intersect <= t_max_z
				{
					break;
				}
			}

			if t_max_x < t_max_y {
				if t_max_x < t_max_z {
					grid_x += step_x;
					if grid_x == just_out_x {
						break;
					};

					t_max_x += t_delta_x;
				} else {
					grid_z += step_z;
					if grid_z == just_out_z {
						break;
					};

					t_max_z += t_delta_z;
				}
			} else {
				if t_max_y < t_max_z {
					grid_y += step_y;
					if grid_y == just_out_y {
						break;
					};

					t_max_y += t_delta_y;
				} else {
					grid_z += step_z;
					if grid_z == just_out_z {
						break;
					};

					t_max_z += t_delta_z;
				}
			}

			cell = self.cell_at(grid_x as usize, grid_y as usize, grid_z as usize);
		}

		hit
	}

	fn fill_worker(
		grid_min: Vector3<f32>,
		cell_size: f32,
		mut cell_list: Vec<(usize, usize, usize)>,
		objects: &Vec<Arc<Object>>,
		tx: Sender<((usize, usize, usize), GridCell)>,
	)
	{
		loop {
			let (x, y, z) = match cell_list.pop() {
				Some(cell) => cell,
				None => break,
			};

			let position = grid_min
				+ Vector3::new(
					cell_size * x as f32,
					cell_size * y as f32,
					cell_size * z as f32,
				);

			tx.send(((x, y, z), GridCell::new(position, cell_size, objects)))
				.unwrap();
		}
	}

	fn get_bbox_corners_in_world_space(objects: &Vec<Arc<Object>>) -> Vec<[Vector4<f32>; 8]>
	{
		let mut corner_vec = Vec::new();

		for object in objects {
			let inv_trans = object.get_transform().try_inverse().unwrap();
			let (min, max) = object.get_bounding_box().get_extents();

			let corners = [
				inv_trans * Vector4::new(min.x, min.y, min.z, 1.0),
				inv_trans * Vector4::new(min.x, min.y, max.z, 1.0),
				inv_trans * Vector4::new(min.x, max.y, min.z, 1.0),
				inv_trans * Vector4::new(min.x, max.y, max.z, 1.0),
				inv_trans * Vector4::new(max.x, min.y, min.z, 1.0),
				inv_trans * Vector4::new(max.x, min.y, max.z, 1.0),
				inv_trans * Vector4::new(max.x, max.y, min.z, 1.0),
				inv_trans * Vector4::new(max.x, max.y, max.z, 1.0),
			];

			corner_vec.push(corners);
		}

		corner_vec
	}

	fn get_min_max_points(corner_vec: &Vec<[Vector4<f32>; 8]>) -> (Vector3<f32>, Vector3<f32>)
	{
		let mut grid_min = Vector3::repeat(f32::INFINITY);
		let mut grid_max = Vector3::repeat(f32::NEG_INFINITY);

		for corners in corner_vec {
			for corner in corners {
				grid_min.x = f32::min(grid_min.x, corner.x);
				grid_min.y = f32::min(grid_min.y, corner.y);
				grid_min.z = f32::min(grid_min.z, corner.z);

				grid_max.x = f32::max(grid_max.x, corner.x);
				grid_max.y = f32::max(grid_max.y, corner.y);
				grid_max.z = f32::max(grid_max.z, corner.z);
			}
		}

		(grid_min, grid_max)
	}

	fn get_average_bbox_volume(corner_vec: &Vec<[Vector4<f32>; 8]>) -> f32
	{
		let mut volume_sum = 0.0;

		for corners in corner_vec {
			let vec_x = corners[4] - corners[0];
			let vec_y = corners[1] - corners[0];
			let vec_z = corners[2] - corners[0];

			let size_x = f32::sqrt(f32::abs(vec_x.dot(&vec_x)));
			let size_y = f32::sqrt(f32::abs(vec_y.dot(&vec_y)));
			let size_z = f32::sqrt(f32::abs(vec_z.dot(&vec_z)));

			volume_sum += size_x * size_y * size_z;
		}

		volume_sum / corner_vec.len() as f32
	}

	fn get_step_directions(&self, ray_direction: Vector4<f32>) -> (i64, i64, i64)
	{
		let step_x = if ray_direction.x >= 0.0 { 1 } else { -1 };
		let step_y = if ray_direction.y >= 0.0 { 1 } else { -1 };
		let step_z = if ray_direction.z >= 0.0 { 1 } else { -1 };

		(step_x, step_y, step_z)
	}

	fn get_step_out_values(&self, ray_direction: Vector4<f32>) -> (i64, i64, i64)
	{
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

		(just_out_x, just_out_y, just_out_z)
	}

	fn get_starting_cell(&self, ray: &Ray) -> Option<(i64, i64, i64)>
	{
		let (grid_x, grid_y, grid_z) = self.get_cell_from_point(ray.origin());

		if grid_x < 0
			|| grid_x >= self.num_cells.x as i64
			|| grid_y < 0
			|| grid_y >= self.num_cells.y as i64
			|| grid_z < 0
			|| grid_z >= self.num_cells.z as i64
		{
			if let Some(t) = self.intersect_grid_bounds(ray) {
				let grid_intersect = ray.origin() + (t * (ray.point() - ray.origin()));
				Some(self.get_cell_from_point(grid_intersect))
			} else {
				None
			}
		} else {
			Some((grid_x, grid_y, grid_z))
		}
	}

	fn get_cell_from_point(&self, ray_origin: Vector4<f32>) -> (i64, i64, i64)
	{
		let offset_x = ray_origin.x - self.position.x;
		let offset_y = ray_origin.y - self.position.y;
		let offset_z = ray_origin.z - self.position.z;

		let grid_x = if math::near_zero(offset_x) {
			0
		} else if math::near_zero(offset_x - self.size.x) {
			self.num_cells.x as i64 - 1
		} else {
			(offset_x / (self.size.x / self.num_cells.x as f32)) as i64
		};

		let grid_y = if math::near_zero(offset_y) {
			0
		} else if math::near_zero(offset_y - self.size.y) {
			self.num_cells.y as i64 - 1
		} else {
			(offset_y / (self.size.y / self.num_cells.y as f32)) as i64
		};

		let grid_z = if math::near_zero(offset_z) {
			0
		} else if math::near_zero(offset_z - self.size.z) {
			self.num_cells.z as i64 - 1
		} else {
			(offset_z / (self.size.z / self.num_cells.z as f32)) as i64
		};

		(grid_x, grid_y, grid_z)
	}

	fn get_max_and_delta(
		&self,
		step: i64,
		ray: &Ray,
		cell_position: Vector4<f32>,
		normal: Vector4<f32>,
	) -> (f32, f32)
	{
		let (first_point_offset, second_point_offset) = if step > 0 {
			(
				Vector4::new(self.cell_size, self.cell_size, self.cell_size, 0.0),
				Vector4::repeat(0.0),
			)
		} else {
			(
				Vector4::repeat(0.0),
				Vector4::new(self.cell_size, self.cell_size, self.cell_size, 0.0),
			)
		};

		let first_point = cell_position + first_point_offset;
		let second_point = cell_position + second_point_offset;

		let la = (ray.origin() - first_point).dot(&normal);
		let lb = (ray.point() - first_point).dot(&normal);

		let t_max = f32::abs(la / (la - lb));

		let la = (ray.origin() - second_point).dot(&normal);
		let lb = (ray.point() - second_point).dot(&normal);

		let t_delta = f32::abs(t_max - (la / (la - lb)));

		(t_max, t_delta)
	}

	fn intersect_grid_bounds(&self, ray: &Ray) -> Option<f32>
	{
		let ray_direction = ray.point() - ray.origin();

		let inv_direction = Vector4::repeat(1.0).component_div(&ray_direction);

		let min = (self.position.x - ray.origin().x) * inv_direction.x;
		let max = (self.position.x + self.size.x - ray.origin().x) * inv_direction.x;

		let (mut t_min, mut t_max) = if inv_direction.x >= 0.0 {
			(min, max)
		} else {
			(max, min)
		};

		let min = (self.position.y - ray.origin().y) * inv_direction.y;
		let max = (self.position.y + self.size.y - ray.origin().y) * inv_direction.y;

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

		let min = (self.position.z - ray.origin().z) * inv_direction.z;
		let max = (self.position.z + self.size.z - ray.origin().z) * inv_direction.z;

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

		if math::far_from_zero_pos(t_min) {
			Some(t_min)
		} else if math::far_from_zero_pos(t_max) {
			Some(t_max)
		} else {
			None
		}
	}

	fn cell_at(&self, x: usize, y: usize, z: usize) -> &GridCell
	{
		&self.cells[x + (y * self.num_cells.x) + (z * self.num_cells.x * self.num_cells.y)]
	}
}

struct GridCell
{
	objects: Vec<Arc<Object>>,
}

impl GridCell
{
	pub fn new(position: Vector3<f32>, size: f32, objects: &Vec<Arc<Object>>) -> Self
	{
		let mut cell = GridCell {
			objects: Vec::new(),
		};

		objects.iter().for_each(|object| {
			let planes = GridCell::get_grid_planes(position, size, object.get_transform());
			let polygons = GridCell::get_bbox_polygons(object);

			// First clip the object bounding box to the grid cell
			// This will find all objects that are within or that intersect a grid cell
			// except for bounding boxes that completely contain a grid cell
			if GridCell::check_polygons_in_cell(&planes, &polygons) {
				cell.objects.push(Arc::clone(object));
			} else {
				let bbox_planes = GridCell::get_bbox_planes(object);
				let grid_point = object.get_transform() * position.insert_row(3, 0.0);

				// If the first check did not find that the object intersected the
				// grid cell we now check if the object bounding box contains a
				// corner of the grid cell. This will catch the one case the above
				// check does not.
				if GridCell::check_point_in_box(&bbox_planes, grid_point) {
					cell.objects.push(Arc::clone(object));
				}
			}
		});

		cell
	}

	pub fn check_hit(&self, ray: &Ray) -> Option<(Hit, &dyn Material)>
	{
		self.objects
			.iter()
			.fold(None, |last_hit, object| -> Option<(Hit, &dyn Material)> {
				if let Some(hit) = object.check_hit(ray) {
					match last_hit {
						Some(last_hit) => {
							if hit.0.intersect < last_hit.0.intersect
								&& math::far_from_zero_pos(hit.0.intersect)
							{
								Some(hit)
							} else {
								Some(last_hit)
							}
						},
						None => Some(hit),
					}
				} else {
					last_hit
				}
			})
	}

	fn check_polygons_in_cell(
		planes: &[(Vector4<f32>, Vector4<f32>); 6],
		polygons: &[[Vector4<f32>; 4]; 6],
	) -> bool
	{
		polygons.iter().any(|polygon| -> bool {
			let output_list = planes
				.iter()
				.fold(Vec::from(polygon.clone()), |input_list, plane| {
					GridCell::clip_polygon_to_plane(&plane, input_list)
				});

			!output_list.is_empty()
		})
	}

	fn clip_polygon_to_plane(
		plane: &(Vector4<f32>, Vector4<f32>),
		input_list: Vec<Vector4<f32>>,
	) -> Vec<Vector4<f32>>
	{
		// Sutherland-Hodgman algorithm

		(0..input_list.len()).fold(Vec::new(), |mut point_list, i| {
			let current_point = input_list[i];
			let prev_point = input_list[(i + input_list.len() - 1) % input_list.len()];

			let la = GridCell::distance_from_plane(current_point, plane);
			let lb = GridCell::distance_from_plane(prev_point, plane);

			if la >= 0.0 {
				if lb < 0.0 {
					point_list.push(GridCell::intersection_from_distances(
						la,
						lb,
						current_point,
						prev_point,
					));
				}

				point_list.push(current_point);
			} else if lb >= 0.0 {
				point_list.push(GridCell::intersection_from_distances(
					la,
					lb,
					current_point,
					prev_point,
				));
			}

			point_list
		})
	}

	fn distance_from_plane(point: Vector4<f32>, plane: &(Vector4<f32>, Vector4<f32>)) -> f32
	{
		(point - plane.0).dot(&plane.1)
	}

	fn intersection_from_distances(
		la: f32,
		lb: f32,
		current_point: Vector4<f32>,
		prev_point: Vector4<f32>,
	) -> Vector4<f32>
	{
		let t = la / (la - lb);

		current_point + (t * (prev_point - current_point))
	}

	fn check_point_in_box(planes: &[(Vector4<f32>, Vector4<f32>); 6], point: Vector4<f32>) -> bool
	{
		planes
			.iter()
			.all(|plane| -> bool { GridCell::distance_from_plane(point, plane) > 0.0 })
	}

	fn get_grid_planes(
		position: Vector3<f32>,
		size: f32,
		transform: Matrix4<f32>,
	) -> [(Vector4<f32>, Vector4<f32>); 6]
	{
		let lower = transform * position.insert_row(3, 1.0);
		let upper = transform * position.add_scalar(size).insert_row(3, 1.0);

		let inverse_transform = transform.try_inverse().unwrap();

		[
			(
				lower,
				math::transform_normals(Vector4::new(1.0, 0.0, 0.0, 0.0), inverse_transform),
			),
			(
				lower,
				math::transform_normals(Vector4::new(0.0, 1.0, 0.0, 0.0), inverse_transform),
			),
			(
				lower,
				math::transform_normals(Vector4::new(0.0, 0.0, 1.0, 0.0), inverse_transform),
			),
			(
				upper,
				math::transform_normals(Vector4::new(-1.0, 0.0, 0.0, 0.0), inverse_transform),
			),
			(
				upper,
				math::transform_normals(Vector4::new(0.0, -1.0, 0.0, 0.0), inverse_transform),
			),
			(
				upper,
				math::transform_normals(Vector4::new(0.0, 0.0, -1.0, 0.0), inverse_transform),
			),
		]
	}

	fn get_bbox_planes(obj: &Object) -> [(Vector4<f32>, Vector4<f32>); 6]
	{
		let (lower, upper) = obj.get_bounding_box().get_extents();

		[
			(lower, Vector4::new(1.0, 0.0, 0.0, 0.0)),
			(lower, Vector4::new(0.0, 1.0, 0.0, 0.0)),
			(lower, Vector4::new(0.0, 0.0, 1.0, 0.0)),
			(upper, Vector4::new(-1.0, 0.0, 0.0, 0.0)),
			(upper, Vector4::new(0.0, -1.0, 0.0, 0.0)),
			(upper, Vector4::new(0.0, 0.0, -1.0, 0.0)),
		]
	}

	fn get_bbox_polygons(obj: &Object) -> [[Vector4<f32>; 4]; 6]
	{
		let (lower, upper) = obj.get_bounding_box().get_extents();

		let points = [
			lower,                                        // Left-Bottom-Back 0
			Vector4::new(upper.x, lower.y, lower.z, 1.0), // Right-Bottom-Back 1
			Vector4::new(upper.x, upper.y, lower.z, 1.0), // Right-Top-Back 2
			Vector4::new(lower.x, upper.y, lower.z, 1.0), // Left-Top-Back 3
			Vector4::new(lower.x, upper.y, upper.z, 1.0), // Left-Top-Front 4
			Vector4::new(lower.x, lower.y, upper.z, 1.0), // Left-Bottom-Front 5
			Vector4::new(upper.x, lower.y, upper.z, 1.0), // Right-Bottom-Front 6
			upper,                                        // Right-Top-Front 7
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
}

unsafe impl Send for GridCell {}
unsafe impl Sync for GridCell {}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn polygon_clip_fully_contained()
	{
		let planes = [
			(
				Vector4::new(-10.0, 0.0, 0.0, 1.0),
				Vector4::new(1.0, 0.0, 0.0, 1.0),
			),
			(
				Vector4::new(10., 0.0, 0.0, 1.0),
				Vector4::new(-1.0, 0.0, 0.0, 1.0),
			),
			(
				Vector4::new(0.0, -10.0, 0.0, 1.0),
				Vector4::new(0.0, 1.0, 0.0, 1.0),
			),
			(
				Vector4::new(0.0, 10.0, 0.0, 1.0),
				Vector4::new(0.0, -1.0, 0.0, 1.0),
			),
			(
				Vector4::new(0.0, 0.0, -10.0, 1.0),
				Vector4::new(0.0, 0.0, 1.0, 1.0),
			),
			(
				Vector4::new(0.0, 0.0, 10.0, 1.0),
				Vector4::new(0.0, 0.0, -1.0, 1.0),
			),
		];

		let points = [
			Vector4::new(-1.0, -1.0, -1.0, 1.0), // Left-Bottom-Back 0
			Vector4::new(1.0, -1.0, -1.0, 1.0),  // Right-Bottom-Back 1
			Vector4::new(1.0, 1.0, -1.0, 1.0),   // Right-Top-Back 2
			Vector4::new(-1.0, 1.0, -1.0, 1.0),  // Left-Top-Back 3
			Vector4::new(-1.0, 1.0, 1.0, 1.0),   // Left-Top-Front 4
			Vector4::new(-1.0, -1.0, 1.0, 1.0),  // Left-Bottom-Front 5
			Vector4::new(1.0, -1.0, 1.0, 1.0),   // Right-Bottom-Front 6
			Vector4::new(1.0, 1.0, 1.0, 1.0),    // Right-Top-Front 7
		];

		let polygons = [
			[points[0], points[3], points[4], points[5]], // Left
			[points[0], points[1], points[6], points[5]], // Bottom
			[points[0], points[1], points[2], points[3]], // Back
			[points[1], points[2], points[7], points[6]], // Right
			[points[2], points[3], points[4], points[7]], // Top
			[points[4], points[5], points[6], points[7]], // Front
		];

		assert!(GridCell::check_polygons_in_cell(&planes, &polygons));
		assert!(GridCell::check_point_in_box(&planes, points[0]));
	}

	#[test]
	fn polygon_clip_fully_contained2()
	{
		let planes = [
			(
				Vector4::new(-1.0, 0.0, 0.0, 1.0),
				Vector4::new(1.0, 0.0, 0.0, 1.0),
			),
			(
				Vector4::new(1., 0.0, 0.0, 1.0),
				Vector4::new(-1.0, 0.0, 0.0, 1.0),
			),
			(
				Vector4::new(0.0, -1.0, 0.0, 1.0),
				Vector4::new(0.0, 1.0, 0.0, 1.0),
			),
			(
				Vector4::new(0.0, 1.0, 0.0, 1.0),
				Vector4::new(0.0, -1.0, 0.0, 1.0),
			),
			(
				Vector4::new(0.0, 0.0, -1.0, 1.0),
				Vector4::new(0.0, 0.0, 1.0, 1.0),
			),
			(
				Vector4::new(0.0, 0.0, 1.0, 1.0),
				Vector4::new(0.0, 0.0, -1.0, 1.0),
			),
		];

		let points = [
			Vector4::new(-10.0, -10.0, -10.0, 1.0), // Left-Bottom-Back 0
			Vector4::new(10.0, -10.0, -10.0, 1.0),  // Right-Bottom-Back 1
			Vector4::new(10.0, 10.0, -10.0, 1.0),   // Right-Top-Back 2
			Vector4::new(-10.0, 10.0, -10.0, 1.0),  // Left-Top-Back 3
			Vector4::new(-10.0, 10.0, 10.0, 1.0),   // Left-Top-Front 4
			Vector4::new(-10.0, -10.0, 10.0, 1.0),  // Left-Bottom-Front 5
			Vector4::new(10.0, -10.0, 10.0, 1.0),   // Right-Bottom-Front 6
			Vector4::new(10.0, 10.0, 10.0, 1.0),    // Right-Top-Front 7
		];

		let polygons = [
			[points[0], points[3], points[4], points[5]], // Left
			[points[0], points[1], points[6], points[5]], // Bottom
			[points[0], points[1], points[2], points[3]], // Back
			[points[1], points[2], points[7], points[6]], // Right
			[points[2], points[3], points[4], points[7]], // Top
			[points[4], points[5], points[6], points[7]], // Front
		];

		assert!(!GridCell::check_polygons_in_cell(&planes, &polygons));
	}
}
