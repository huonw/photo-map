extern crate cogset;

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

        let speed = sphere_dist / time; // km/s
        //println!("{:8} {:8} {:8}", sphere_dist, time, speed);
        let speed = speed * 3.6; // km/h
        let speed_dist = speed * speed_factor;

        sphere_dist * sphere_dist + time_dist * time_dist + speed_dist * speed_dist
    }
}

use std::ops::{Range, RangeFrom};
use std::iter::{Enumerate, Rev, Zip};
use std::slice::Iter;

struct PointSet<'a> {
    points: &'a [Point],
    time_factor: f64,
    speed_factor: f64,
}

impl<'a> cogset::Points for PointSet<'a> {
    type Point = usize;
}
impl<'a> cogset::ListPoints for PointSet<'a> {
    type AllPoints = Range<usize>;

    fn all_points(&self) -> Range<usize> {
        0..self.points.len()
    }
}
impl<'a> cogset::RegionQuery for PointSet<'a> {
    type Neighbours = Neighbours<'a>;

    fn neighbours(&self, &point: &usize, max_dist2: f64) -> Neighbours<'a> {
        Neighbours {
            state: State::Prefix,
            prefix: self.points[..point].iter().enumerate().rev(),
            suffix: (point..).zip(self.points[point..].iter()),
            point: &self.points[point],
            time_factor: self.time_factor,
            speed_factor: self.speed_factor,
            max_dist2: max_dist2,
            count: 0,
        }
    }
}

enum State { Prefix, Suffix, End }
impl State {
    fn transition(&mut self) {
        *self = match *self {
            State::Prefix => State::Suffix,
            _ => State::End
        }
    }
}
struct Neighbours<'a> {
    state: State,
    prefix: Rev<Enumerate<Iter<'a, Point>>>,
    suffix: Zip<RangeFrom<usize>, Iter<'a, Point>>,
    point: &'a Point,
    time_factor: f64,
    speed_factor: f64,
    max_dist2: f64,
    count: usize
}

impl<'a> Iterator for Neighbours<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        loop {
            let item = match self.state {
                State::Prefix => self.prefix.next(),
                State::Suffix => self.suffix.next(),
                State::End => return None,
            };
            let (i, p) = match item {
                Some(t) => t,
                None => {
                    //println!("{:5}: broke after {} (a)", self.point.id, self.count);
                    self.count = 0;
                    self.state.transition();
                    continue
                }
            };
            let exceeded =
                self.point.dist2_lower_bound(p, self.time_factor,
                                             self.speed_factor) > self.max_dist2 ||
                self.point.dist2(p, self.time_factor, self.speed_factor) > self.max_dist2;

            if !exceeded {
                self.count += 1;
                return Some(i);
            }

            // we've set-up the data structure such that the lower
            // bound is a lower bound for all further things along
            // this *fix, so if it is violated here we can
            // shortcircuit.
            self.count = 0;
            self.state.transition();
        }
    }
}

pub fn cluster_points(points: &[Point],
                      time_factor: f64, speed_factor: f64,
                      max_dist: f64, min_points: usize) -> Vec<Vec<i64>> {
    println!("{}", max_dist * max_dist);
    let set = PointSet {
        points: points,
        time_factor: time_factor,
        speed_factor: speed_factor,
    };

    let mut dbscan = cogset::Dbscan::new(set, max_dist * max_dist, min_points);

    let mut clusters = dbscan.by_ref()
        .map(|v| v.iter().map(|idx| points[*idx].id).collect())
        .collect::<Vec<_>>();

    clusters.extend(dbscan.noise_points().iter().map(|idx| vec![points[*idx].id]));

    clusters
}
