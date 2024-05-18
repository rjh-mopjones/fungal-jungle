
use rand::{Rng, SeedableRng};
#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

pub fn generate(height: usize, width: usize, num_points: usize, num_relaxations: usize, seed: u64, contrast: f64, shift: f64) -> Vec<Vec<f64>> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut points = Vec::with_capacity(num_points);

    // Generate random points
    for _ in 0..num_points {
        let x = rng.gen_range(0.0..height as f64);
        let y = rng.gen_range(0.0..width as f64);
        points.push(Point { x, y });
    }

    // Iteratively apply Lloyd's Algorithm
    for _ in 0..num_relaxations {
        // Compute cell centroids
        let mut centroids = vec![Point { x: 0.0, y: 0.0 }; num_points];
        let mut counts = vec![0; num_points];

        for i in 0..height {
            for j in 0..width {
                let mut closest = 0;
                let mut closest_distance = f64::MAX;
                for (idx, point) in points.iter().enumerate() {
                    let distance = point.distance(&Point { x: i as f64, y: j as f64 });
                    if distance < closest_distance {
                        closest_distance = distance;
                        closest = idx;
                    }
                }
                centroids[closest].x += i as f64;
                centroids[closest].y += j as f64;
                counts[closest] += 1;
            }
        }

        // Move points to centroids
        for i in 0..num_points {
            if counts[i] > 0 {
                points[i].x = centroids[i].x / counts[i] as f64;
                points[i].y = centroids[i].y / counts[i] as f64;
            }
        }
    }

    // Compute noise map
    let mut noise_map = vec![vec![0.0; width]; height];
    for i in 0..height {
        for j in 0..width {
            let mut distances = Vec::with_capacity(num_points);
            for point in &points {
                distances.push(point.distance(&Point { x: i as f64, y: j as f64 }));
            }
            distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
            noise_map[i][j] = distances[1]; // Use distance to the second closest point
        }
    }
    // Normalize noise map
    let mut min_value = f64::MAX;
    let mut max_value = f64::MIN;
    for i in 0..height {
        for j in 0..width {
            min_value = min_value.min(noise_map[i][j]);
            max_value = max_value.max(noise_map[i][j]);
        }
    }
    let range = max_value - min_value;
    for i in 0..height {
        for j in 0..width {
            noise_map[i][j] = (((noise_map[i][j] - min_value) / range) * contrast + shift)
                .min(1.0).max(0.0);
        }
    }

    // Invert noise map
    for i in 0..height {
        for j in 0..width {
            noise_map[i][j] = 1.0 - noise_map[i][j];
        }
    }
    noise_map
}