#![feature(exit_status)]

extern crate rusqlite;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate getopts;

extern crate photo_map;

use std::collections::HashMap;
use std::env;
use std::io;
use std::io::prelude::*;
use std::path::Path;

struct Point {
    id: i64,
    latitude: f64,
    longitude: f64,
    gps_timestamp: i64
}

const EARTH_RADIUS: f64 = 6_371.0;
impl Point {
    fn dist2_lower_bound(&self, other: &Point, time_factor: f64) -> f64 {
        let time = (self.gps_timestamp - other.gps_timestamp) as f64 * time_factor;
        time * time
    }
    fn dist2(&self, other: &Point, time_factor: f64) -> f64 {
        let sphere_dist = photo_map::sphere_distance(EARTH_RADIUS,
                                                     self.latitude, other.latitude,
                                                     self.longitude, other.longitude);

        let time = (self.gps_timestamp - other.gps_timestamp) as f64 * time_factor;

        sphere_dist * sphere_dist + time * time
        //dlat * dlat + dlon * dlon + time * time
    }
}

fn main() {
    env_logger::init().unwrap();

    let mut args = env::args();
    let program = args.next().unwrap();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this message");
    opts.reqopt("d", "database", "SQLite database to insert records into", "FILE");
    opts.optopt("t", "time-factor", "how many metres one second counts for", "NUM");

    let matches = match opts.parse(args) {
        Ok(m) => m,
        Err(e) => {
            writeln!(&mut io::stderr(), "invalid arguments: {}", e).unwrap();
            env::set_exit_status(1);
            return
        }
    };

    if matches.opt_present("h") {
        println!("{}", opts.usage(&format!("Usage: {} [options]", program)));
        return
    }

    let db_file = &matches.opt_str("d").unwrap();
    let db_file = Path::new(db_file);
    let conn = rusqlite::SqliteConnection::open_with_flags(db_file,
                                                           rusqlite::SQLITE_OPEN_READ_ONLY)
        .unwrap();

    let mut query = conn.prepare("SELECT ROWID, latitude, longitude,
                                         gps_timestamp
                                  FROM positions
                                  ORDER BY gps_timestamp").unwrap();
    let mut points = query.query(&[]).unwrap().map(|r| r.unwrap())
        .map(|r| Point {
            id: r.get(0), latitude: r.get(1), longitude: r.get(2), gps_timestamp: r.get(3)
        })
        .collect::<Vec<_>>();
    //points.truncate(2000);
    let mut outer = Vec::with_capacity(points.len());
    let mut total_counts = 0;
    let mut max_count = 0;
    for (i, point) in points.iter().enumerate() {
        let mut count = 0;
        let mut inner = Vec::with_capacity(points.len());
        const MAX_DIST: f64 = 1_000.0;
        const TIME_FACTOR: f64 = 1_000.0 / (60.0 * 60.0 * 24.0);
        count += search_points(&mut inner, point, points[..i].iter().rev(), MAX_DIST, TIME_FACTOR);
        count += search_points(&mut inner, point, points[i..].iter(), MAX_DIST, TIME_FACTOR);
        inner.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        outer.push((point.id, inner));
        total_counts += count;
        if count > max_count {
            max_count = count;
            println!("{} {} {}", count, point.latitude, point.longitude);
        }
    }
    outer.sort_by(|a, b| a.0.cmp(&b.0));
    println!("{}", total_counts as f64 / points.len() as f64);
    println!("{}", max_count);
}

fn search_points<'a, I: Iterator<Item = &'a Point>>(v: &mut Vec<(f64, i64)>,
                                                    point: &Point, mut points: I,
                                                    max_dist: f64, time_factor: f64) -> usize {
    let mut count = 0;
    let max_dist2 = max_dist * max_dist;
    for point2 in points {
        let dist2_lower_bound = point.dist2_lower_bound(point2, time_factor);
        if dist2_lower_bound > max_dist2 { break }
        let dist2 = point.dist2(point2, time_factor);
        if dist2 > max_dist2 { break }
        v.push((dist2, point2.id));
        count += 1;
    }
    count
}
