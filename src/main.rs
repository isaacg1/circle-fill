use gif::{Encoder, Frame};
use image::{ImageBuffer, RgbImage};
use noisy_float::prelude::*;
use rand::prelude::*;

use std::f64::consts::TAU;
use std::fs::File;
use std::time::SystemTime;

const DEBUG: bool = false;
const GIF: bool = false;

type Color = [u8; 3];
const GRAY: Color = [128, 128, 128];
type Location = [usize; 2];
#[derive(Clone, Copy, Debug)]
struct Record {
    color: Color,
    location: Location,
}

fn make_image(
    size: usize,
    num_samples: usize,
    num_seeds: usize,
    circle_frac: f64,
    timeout: u64,
    seed: u64,
) -> RgbImage {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut grid: Vec<Vec<Option<Color>>> = vec![vec![None; size]; size];
    let mut placed: Vec<Record> = vec![];
    let mut num_placed: usize = 0;
    let mut misses = 0;
    let mut max_misses = 0;
    let start = SystemTime::now();
    let mut cur_perc = 0;
    let filename = format!(
        "{}-{}-{}-{}-{}-{}.gif",
        size, num_samples, num_seeds, circle_frac, timeout, seed
    );
    let mut image = if GIF {
        Some(File::create(filename).expect("Made gif"))
    } else {
        None
    };
    let mut encoder = if GIF { Some(Encoder::new(
        image.as_mut().expect("present"),
        size as u16,
        size as u16,
        &[],
    )
    .expect("Made encoder"))
    } else {
        None
    };
    'outer: while misses < timeout {
        if GIF {
            let perc = (100.0 * placed.len() as f64 / size.pow(2) as f64) as u64;
            if perc > cur_perc {
                cur_perc = perc;
                let mut writeable: Vec<u8> = grid
                    .iter()
                    .flat_map(|row| row.iter().flat_map(|cell| cell.unwrap_or(GRAY).into_iter()))
                    .collect();
                let frame = Frame::from_rgb(size as u16, size as u16, &mut *writeable);
                encoder.as_mut().expect("still present").write_frame(&frame).expect("Written");
            }
        }
        if num_placed.is_power_of_two() {
            println!(
                "{}% {}% {} {}",
                (100.0 * placed.len() as f64 / size.pow(2) as f64) as u64,
                (100.0 * num_placed as f64 / size.pow(2) as f64) as u64,
                max_misses,
                start.elapsed().expect("monotonic").as_secs()
            )
        }
        num_placed += 1;
        misses += 1;
        max_misses = max_misses.max(misses);
        // Generate random color
        let color: Color = rng.random();
        // If below num_seeds: Place randomly.
        if num_placed <= num_seeds {
            let loc = [rng.random_range(0..size), rng.random_range(0..size)];
            if grid[loc[0]][loc[1]].is_none() {
                grid[loc[0]][loc[1]] = Some(color);
                placed.push(Record {
                    color,
                    location: loc,
                });
                misses = 0;
            }
            continue 'outer;
        }
        if DEBUG {
            std::io::stdin()
                .read_line(&mut String::new())
                .expect("pause");
        }
        // Sample num_samples from vec of placed colors, without replacement.
        let mut samples: Vec<Record> = placed
            .choose_multiple(&mut rng, num_samples)
            .copied()
            .collect();
        // Find 3 closest colors
        samples.sort_by_key(|record| {
            let oth_color = record.color;
            color
                .into_iter()
                .zip(oth_color)
                .map(|(c1, c2)| (c1 as i64 - c2 as i64).pow(2))
                .sum::<i64>()
        });
        // Find circumcenter and radius
        // https://en.wikipedia.org/wiki/Circumcircle#Cartesian_coordinates
        let a = samples[0].location;
        let b = samples[1].location;
        let c = samples[2].location;
        let ax = a[0] as isize;
        let ay = a[1] as isize;
        let bx = b[0] as isize;
        let by = b[1] as isize;
        let cx = c[0] as isize;
        let cy = c[1] as isize;
        let a2 = ax.pow(2) + ay.pow(2);
        let b2 = bx.pow(2) + by.pow(2);
        let c2 = cx.pow(2) + cy.pow(2);
        let sx = (a2 * by + b2 * cy + c2 * ay - a2 * cy - b2 * ay - c2 * by) / 2;
        let sy = (ax * b2 + bx * c2 + cx * a2 - ax * c2 - bx * a2 - cx * b2) / 2;
        let da = ax * by + bx * cy + cx * ay - ax * cy - bx * ay - cx * by;
        if da == 0 {
            continue 'outer;
        }
        let center = [sx / da, sy / da];
        // Find middle vertex, second vertex
        let ang_a = ((center[0] - ax) as f64).atan2((center[1] - ay) as f64);
        let ang_b = ((center[0] - bx) as f64).atan2((center[1] - by) as f64);
        let ang_c = ((center[0] - cx) as f64).atan2((center[1] - cy) as f64);
        let normed = &|f: f64| {
            [f + TAU, f, f - TAU]
                .map(|f| f.abs())
                .into_iter()
                .min_by_key(|&f| n64(f))
                .expect("present")
        };
        let diff_ab = normed(ang_a - ang_b);
        let diff_ac = normed(ang_a - ang_c);
        let diff_bc = normed(ang_b - ang_c);
        let (middle, second) = match (diff_ab <= diff_ac, diff_ab <= diff_bc, diff_ac <= diff_bc) {
            // a, b: true, true, true
            (true, true, true) => ([ax, ay], [bx, by]),
            // a, c: false, true, true
            (false, true, true) => ([ax, ay], [cx, cy]),
            // b, a: true, true, false
            (true, true, false) => ([bx, by], [ax, ay]),
            // b, c: true, false, false
            (true, false, false) => ([bx, by], [cx, cy]),
            // c, a: false, false, true
            (false, false, true) => ([cx, cy], [ax, ay]),
            // c, b: false, false, false
            (false, false, false) => ([cx, cy], [bx, by]),
            // Fails transitivity: ab <= ac <= bc, but ab !<= bc
            (true, false, true) => unreachable!(),
            // Fails transitivity: ab !<= ac !<= bc, but ab <= bc
            (false, true, false) => unreachable!(),
        };
        // Walk around the circle from middle towards second until empty vertex is reached or loop around.
        if DEBUG {
            println!("center {:?}", center);
        }
        let mut cur = middle;
        let ang_second = ((center[0] - second[0]) as f64).atan2((center[1] - second[1]) as f64);
        let mut next = [
            [cur[0] + 1, cur[1] + 1],
            [cur[0] + 1, cur[1]],
            [cur[0] + 1, cur[1] - 1],
            [cur[0], cur[1] + 1],
            [cur[0], cur[1] - 1],
            [cur[0] - 1, cur[1] + 1],
            [cur[0] - 1, cur[1]],
            [cur[0] - 1, cur[1] - 1],
        ]
        .into_iter()
        .filter(|&p_next| p_next != second)
        .min_by_key(|p_next| {
            let ang_p = ((center[0] - p_next[0]) as f64).atan2((center[1] - p_next[1]) as f64);
            n64(normed(ang_second - ang_p))
        })
        .expect("found");
        let sq_radius = (center[0] - middle[0]).pow(2) + (center[1] - middle[1]).pow(2);
        let radius = (sq_radius as f64).sqrt();
        if DEBUG {
            println!("cur {:?} next {:?}", cur, next);
        }
        for _ in 0..(radius * TAU * circle_frac) as u64 {
            let loc = [
                ((next[0] % size as isize) + size as isize) as usize % size,
                ((next[1] % size as isize) + size as isize) as usize % size,
            ];
            if DEBUG {
                println!("loc {:?}", loc);
            }
            if grid[loc[0]][loc[1]].is_none() {
                grid[loc[0]][loc[1]] = Some(color);
                placed.push(Record {
                    color,
                    location: loc,
                });
                misses = 0;
                continue 'outer;
            }
            assert_ne!(cur, next);
            let new0 = next[0] * 2 - cur[0];
            let new1 = next[1] * 2 - cur[1];
            let new_candidates = if cur[0] == next[0] {
                [[next[0] + 1, new1], [next[0], new1], [next[0] - 1, new1]]
            } else if cur[1] == next[1] {
                [[new0, next[1] + 1], [new0, next[1]], [new0, next[1] - 1]]
            } else {
                [[new0, new1], [new0, next[1]], [next[0], new1]]
            };
            if DEBUG {
                println!("new_candidates {:?}", new_candidates);
            }
            let new = new_candidates
                .into_iter()
                .min_by_key(|p_new| {
                    ((center[0] - p_new[0]).pow(2) + (center[1] - p_new[1]).pow(2) - sq_radius)
                        .abs()
                })
                .expect("new found");
            if DEBUG {
                println!("new {:?}", new_candidates);
            }
            cur = next;
            next = new;
        }
        // If nothing found, skip.
    }
    // Convert to final image
    let mut img: RgbImage = ImageBuffer::new(size as u32, size as u32);
    for (x, row) in grid.iter().enumerate() {
        for (y, &color) in row.iter().enumerate() {
            img.put_pixel(
                x as u32,
                y as u32,
                image::Rgb(color.unwrap_or([128, 128, 128])),
            )
        }
    }
    img
}

fn main() {
    let size = 1000;
    let num_samples = 100;
    let num_seeds = 20;
    let circle_frac = 0.01;
    let timeout = 10000;
    let seed = 0;
    let filename = format!(
        "{}-{}-{}-{}-{}-{}.png",
        size, num_samples, num_seeds, circle_frac, timeout, seed
    );
    println!("{}", filename);
    let image = make_image(size, num_samples, num_seeds, circle_frac, timeout, seed);
    image.save(&filename).expect("saved");
}
