#![feature(exit_status)]

extern crate rusqlite;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate getopts;
extern crate rustc_serialize;

use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

#[derive(RustcEncodable)]
struct Summary {
    ids: Vec<i64>,
    coords: Vec<(f64, f64)>,
    times: Vec<(i64, i64)>,
}
#[derive(RustcEncodable)]
struct Cluster {
    id: i64,
    coords: Vec<(f64, f64)>,
    times: Vec<i64>,
    mean_time: i64,
}

const DEFAULT_MIN_POINTS: usize = 5;

fn main() {
    env_logger::init().unwrap();

    let mut args = env::args();
    let program = args.next().unwrap();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this message");
    opts.reqopt("d", "database", "SQLite database to insert records into", "FILE");
    opts.optopt("m", "min-points", "minimum number of points to count as a real cluster", "NUM");
    opts.reqopt("s", "summary", "file to print summary to", "FILE");
    opts.reqopt("c", "clusters", "flie to print clusters to", "FILE");

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

    let min_points = match matches.opt_str("m").map(|s| s.parse()) {
        None => DEFAULT_MIN_POINTS,
        Some(Ok(m)) => m,
        Some(Err(e)) => {
            panic!("invalid argument -m: {}", e)
        }
    };

    let mut summary_file = File::create(&matches.opt_str("s").unwrap()).unwrap();
    let mut clusters_file = File::create(&matches.opt_str("c").unwrap()).unwrap();


    let db_file = &matches.opt_str("d").unwrap();
    let db_file = Path::new(db_file);
    let conn = rusqlite::SqliteConnection::open_with_flags(db_file,
                                                           rusqlite::SQLITE_OPEN_READ_ONLY)
        .unwrap();

    let mut clusters = conn.prepare(
        "SELECT clusters.ROWID, clusters.latitude, clusters.longitude,
                MIN(positions.gps_timestamp), MAX(positions.gps_timestamp),
                clusters.gps_timestamp, num_points
         FROM clusters
         INNER JOIN positions ON cluster_id = clusters.ROWID
         GROUP BY clusters.ROWID
         ORDER BY clusters.gps_timestamp")
                           .unwrap()
                           .query(&[])
                           .unwrap()
                           .map(|r| r.unwrap())
                           .map(|r| (r.get::<i64>(0),
                                     r.get::<f64>(1),
                                     r.get::<f64>(2),
                                     r.get::<i64>(3),
                                     r.get::<i64>(4),
                                     r.get::<i64>(5),
                                     r.get::<i64>(6)))
                           .collect::<Vec<_>>();

    let mut query = conn.prepare("SELECT latitude, longitude, gps_timestamp
                                  FROM positions
                                  WHERE cluster_id = ?
                                  ORDER BY gps_timestamp").unwrap();

    let mut summary = Summary { ids: vec![], coords: vec![], times: vec![] };
    let mut cluster_data = vec![];
    for c in &clusters {
        if c.6 >= min_points as i64 {
            summary.ids.push(c.0);
            summary.coords.push((c.1, c.2));
            summary.times.push((c.3, c.4));
        }
        let mut coords = vec![];
        let mut times = vec![];
        for r in query.query(&[&c.0]).unwrap() {
            let r = r.unwrap();
            coords.push((r.get(0), r.get(1)));
            times.push(r.get(2));
        }

        cluster_data.push(Cluster {
            id: c.0,
            coords: coords,
            times: times,
            mean_time: c.5
        })
    }

    write!(&mut summary_file, "{}", rustc_serialize::json::as_json(&summary)).unwrap();
    write!(&mut clusters_file, "{}", rustc_serialize::json::as_json(&cluster_data)).unwrap();
}
