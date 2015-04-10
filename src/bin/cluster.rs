#![feature(exit_status)]

extern crate rusqlite;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate getopts;
extern crate chrono;
extern crate rand;

extern crate photo_map;

use rand::Rng;

use std::collections::HashMap;
use std::env;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use photo_map::Point;

const SECONDS_PER_DAY: f64 = 24.0 * 60.0 * 60.0;
const DEFAULT_TIME_FACTOR: f64 = 20.0 / SECONDS_PER_DAY;
const DEFAULT_SPEED_FACTOR: f64 = 0.0;
const DEFAULT_DIST: f64 = 20.0;
const DEFAULT_MIN_POINTS: usize = 5;

fn main() {
    env_logger::init().unwrap();

    let mut args = env::args();
    let program = args.next().unwrap();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this message");
    opts.reqopt("d", "database", "SQLite database to insert records into", "FILE");
    opts.optopt("t", "time-factor", "how many kilometres one day counts for", "NUM");
    opts.optopt("s", "speed-factor", "number of hours to spread speed over", "NUM");
    opts.optopt("x", "dist", "epsilon to use for dbscan", "NUM");
    opts.optopt("m", "min-points", "minimum number of points in a cluster", "NUM");
    opts.optflag("n", "dry-run", "don't edit the database");

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

    let time_factor = match matches.opt_str("t").map(|s| s.parse::<f64>()) {
        None => DEFAULT_TIME_FACTOR,
        Some(Ok(t)) => t / SECONDS_PER_DAY,
        Some(Err(e)) => {
            panic!("invalid argument -t: {}", e)
        }
    };
    let speed_factor = match matches.opt_str("s").map(|s| s.parse::<f64>()) {
        None => DEFAULT_SPEED_FACTOR,
        Some(Ok(t)) => t,
        Some(Err(e)) => {
            panic!("invalid argument -s: {}", e)
        }
    };
    let dist = match matches.opt_str("x").map(|s| s.parse()) {
        None => DEFAULT_DIST,
        Some(Ok(d)) => d,
        Some(Err(e)) => {
            panic!("invalid argument -x: {}", e)
        }
    };
    let min_points = match matches.opt_str("m").map(|s| s.parse()) {
        None => DEFAULT_MIN_POINTS,
        Some(Ok(m)) => m,
        Some(Err(e)) => {
            panic!("invalid argument -m: {}", e)
        }
    };

    let dry = matches.opt_present("n");

    let db_file = &matches.opt_str("d").unwrap();
    let db_file = Path::new(db_file);
    let flags = if dry {
        rusqlite::SQLITE_OPEN_READ_ONLY
    } else {
        rusqlite::SQLITE_OPEN_READ_WRITE
    };
    let conn = rusqlite::SqliteConnection::open_with_flags(db_file, flags).unwrap();
    let trans = conn.transaction().unwrap();

    let mut rng = rand::thread_rng();
    let mut query = conn.prepare("SELECT ROWID, latitude, longitude,
                                         gps_timestamp, camera_timestamp,
                                         near_home
                                  FROM positions
                                  GROUP BY true_latitude, true_longitude, gps_timestamp
                                  ORDER BY gps_timestamp
                                ").unwrap();
    let points = query.query(&[]).unwrap().map(|r| r.unwrap())
        .filter_map(|r| {
            let near_home = r.get::<i64>(5) == 1;

            if near_home && rng.gen::<f64>() >= 0.02 {
                None
            } else {
                Some(Point {
                    id: r.get(0), latitude: r.get(1), longitude: r.get(2),
                    gps_timestamp: r.get(3), camera_timestamp: r.get(4),
                })
            }
        })
        .collect::<Vec<_>>();

    let by_id = points.iter().map(|p| (p.id, p)).collect::<HashMap<_, _>>();

    if !dry {
        conn.execute("DELETE FROM clusters", &[]).unwrap();
        conn.execute("UPDATE positions SET cluster_id = NULL", &[]).unwrap();
    }
    let mut insert_cluster = conn.prepare(
        "INSERT INTO clusters (latitude, longitude, camera_timestamp, gps_timestamp, num_points)
         VALUES (?, ?, ?, ?, ?)").unwrap();
    let mut update_cluster =
        conn.prepare("UPDATE positions SET cluster_id = ? WHERE ROWID = ?").unwrap();

    let clusters = photo_map::cluster_points(&points, time_factor, speed_factor, dist, min_points);
    println!("total: points {}, clusters {}", points.len(), clusters.len());
    for (i, c) in clusters.iter().enumerate() {
        let mut lat = 0.0;
        let mut lon = 0.0;
        let mut gps_time = 0 as f64;
        let mut camera_time = 0 as f64;

        for id in c {
            let p = by_id[id];
            lat += p.latitude;
            lon += p.longitude;
            camera_time += p.camera_timestamp as f64;
            gps_time += p.gps_timestamp as f64;
        }

        let len = c.len();
        let len_ = len as f64;
        let ave_lat = lat / len_;
        let ave_lon = lon / len_;
        let ave_ctime = (camera_time / len_) as i64;
        let ave_gtime = (gps_time / len_) as i64;

        if !dry {
            insert_cluster.execute(&[&ave_lat, &ave_lon, &ave_ctime, &ave_gtime, &(len as i64)])
                .unwrap();
            let cluster_id = conn.last_insert_rowid();
            for id in c {
                update_cluster.execute(&[&cluster_id, id]).unwrap();
            }
        }
        println!("{:3}, count: {:4}, {:12.8} {:12.8}, {:.0}",
                 i + 1,
                 len, ave_lat, ave_lon,
                 chrono::NaiveDateTime::from_timestamp(ave_ctime, 0));
    }

    trans.commit().unwrap();
}
