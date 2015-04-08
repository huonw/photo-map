pub fn sphere_distance(r: f64, lat1: f64, lat2: f64, lon1: f64, lon2: f64) -> f64 {
    fn hav(theta: f64) -> f64 {
        let s = (theta / 2.0).sin();
        s * s
    }
    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();
    let lon1 = lon1.to_radians();
    let lon2 = lon2.to_radians();
    let dlat = lat1 - lat2;
    let dlon = lon1 - lon2;
    2.0 * r * (hav(dlat) + lat1.cos() * lat2.cos() * hav(dlon)).sqrt().asin()
}

#[derive(PartialEq, Debug)]
pub struct Point {
    pub id: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub gps_timestamp: i64,
    pub camera_timestamp: i64,
}

const EARTH_RADIUS: f64 = 6_371.0;
impl Point {
    fn dist2_lower_bound(&self, other: &Point,
                         time_factor: f64,
                         _speed_factor: f64) -> f64 {
        let time = (self.gps_timestamp - other.gps_timestamp) as f64 * time_factor;
        time * time
    }
    fn dist2(&self, other: &Point, time_factor: f64, speed_factor: f64) -> f64 {
        let sphere_dist = sphere_distance(EARTH_RADIUS,
                                          self.latitude, other.latitude,
                                          self.longitude, other.longitude);
        //println!("sphere_dist: {}", sphere_dist);
        let time = (self.gps_timestamp - other.gps_timestamp) as f64;
        let time_dist = time * time_factor;

        let speed = sphere_dist / time; // m/s
        let speed = speed * 3.6; // km/h
        let speed_dist = speed * speed_factor;

        sphere_dist * sphere_dist + time_dist * time_dist + speed_dist * speed_dist
        //dlat * dlat + dlon * dlon + time * time
    }
}

use std::collections::{HashSet};

pub fn dbscan(points: &[Point],
              time_factor: f64, speed_factor: f64,
              max_dist: f64, min_points: usize) -> Vec<Vec<i64>> {
    let mut clusters = vec![];
    let mut visited = HashSet::new();
    let mut in_cluster = HashSet::new();

    let max_dist2 = max_dist * max_dist;

    for (i, point) in points.iter().enumerate() {
        if !visited.insert(point.id) {
            continue
        }

        //println!("new cluster: {:?}", point);
        let mut nbrs = vec![];
        neighbours(&mut nbrs, i, point, points, max_dist2, time_factor, speed_factor);

        let mut cluster = vec![point.id];
        in_cluster.insert(point.id);

        if nbrs.len() >= min_points {
            let mut idx = 0;
            while idx < nbrs.len() {
                let i2 = nbrs[idx].2;
                let p2 = &points[i2];
                if visited.insert(p2.id) {
                    let old_len = nbrs.len();
                    neighbours(&mut nbrs, i2, p2, points, max_dist2, time_factor, speed_factor);
                    if nbrs.len() - old_len < min_points {
                        //println!("not enough new");
                        // undo: the new point doesn't have enough close
                        nbrs.truncate(old_len)
                    }
                }
                if in_cluster.insert(p2.id) {
                    cluster.push(p2.id)
                }

                idx += 1;
            }
        }

        clusters.push(cluster);
    }

    clusters
}

/*
struct Context<'a> {
    points: &[Point],
    time_factor: f64,
    max_dist: f64,
    max_dist2: f64,
    min_points: usize,
    metadata: HashMap<i64, (bool, Option<f64>)>,
    ordered: Vec<i64>,
}
pub fn optics(points: &[Point],
              time_factor: f64,
              max_dist: f64, min_points: usize) -> Vec<Vec<i64>> {
    let mut metadata =
        points.iter()
              .map(|p| (p.id, (false, None::<f64>)))
        .collect::<HashMap<_, _>>();

    let mut cx = Context {
        points: points,
        time_factor: time_factor,
        max_dist: max_dist,
        max_dist2: max_dist * max_dist,
        min_points: min_points,
        metadata: metadata,
        ordered: vec![],
    };

    for (i, p) in points.iter().enumerate() {
        {
            let elem = cx.metadata.get_mut(&p.id).unwrap();
            // already processed
            if elem.0 { continue }
            elem.0 = true;
        }
    }

}

impl<'a> Context<'a> {
    fn process_point(&mut self, i: usize, point: &Point, inner: bool) {
        let mut neighbours = vec![];
        search_points(&mut neighbours, point, self.points[..i].iter().rev(),
                      self.max_dist2, self.time_factor);
        search_points(&mut neighbours, point, self.points[i..].iter(),
                      self.max_dist2, self.time_factor);

        if neighbours.len() < self.min_points {
            return
        }

        neighbours.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let core_dist2 = neighbours[self.min_points].0;
        let mut seeds = BinaryHeap::new();

    }

    fn update(&mut self, seeds: &mut BinaryHeap<)
}

*/
fn search_points<'a, T, I: Iterator<Item = (T, &'a Point)>>(v: &mut Vec<(f64, i64, T)>,
                                                            point: &Point, points: I,
                                                            max_dist2: f64,
                                                            time_factor: f64,
                                                            speed_factor: f64)
                                                            -> usize
{
    let mut count = 0;
    for (x, point2) in points {
        let dist2_lower_bound = point.dist2_lower_bound(point2, time_factor, speed_factor);
        if dist2_lower_bound > max_dist2 { break }

        let dist2 = point.dist2(point2, time_factor, speed_factor);
        if dist2 > max_dist2 { break }
        v.push((dist2, point2.id, x));
        count += 1;
    }
    count
}

fn neighbours(neighbours: &mut Vec<(f64, i64, usize)>,
              i: usize, point: &Point, points: &[Point],
              max_dist2: f64, time_factor: f64, speed_factor: f64) {
    search_points(neighbours,
                  point,
                  points[..i].iter().enumerate().rev(),
                  max_dist2, time_factor, speed_factor);
    search_points(neighbours,
                  point,
                  (i..).zip(points[i..].iter()),
                  max_dist2, time_factor, speed_factor);
}
