use na::Vector4;
use std::sync::atomic::AtomicUsize;
use std::cell::Cell;

static NEXT_THREAD_ID: AtomicUsize = AtomicUsize::new(0);

thread_local! {
	static NEXT_RAY_ID: Cell<u64> = Cell::new(0);

	// This is a bit precarious, could conceivably break if something were
	// to use a ray before the main tracing portion of the program started.
	// I should come up with something more robust.
	static THREAD_ID: usize = NEXT_THREAD_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

pub struct Ray
{
	id: u64,
	thread: usize,
	pub point: Vector4<f32>,
	pub origin: Vector4<f32>,
}

impl Ray {
	pub fn new(origin: &Vector4<f32>, point: &Vector4<f32>) -> Self
	{
		let thread = THREAD_ID.with(|thread_id| *thread_id);

		let id = NEXT_RAY_ID.with(|next_ray_id| {
			next_ray_id.replace(next_ray_id.get() + 1)
		});

		Ray {
			id: id,
			thread: thread,
			point: *point,
			origin: *origin
		}
	}

	pub fn get_thread(&self) -> usize {
		self.thread
	}

	pub fn get_id(&self) -> u64 {
		self.id
	}
}

pub struct Hit
{
	pub intersect: f32,
	pub normal: Vector4<f32>,
	pub uv: (f32, f32),
}
