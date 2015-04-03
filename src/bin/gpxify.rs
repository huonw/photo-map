#![feature(exit_status)]

extern crate rusqlite;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate getopts;

use std::env;
use std::io;
use std::io::prelude::*;
use std::path::Path;

fn main() {
    env_logger::init().unwrap();

    let mut args = env::args();
    let program = args.next().unwrap();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this message");
    opts.reqopt("d", "database", "SQLite database to insert records into", "FILE");

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

    let mut clusters = conn.prepare(
        "SELECT ROWID, latitude, longitude,
                strftime('%Y-%m-%dT%H:%M:%SZ', gps_timestamp, 'unixepoch'),
                num_points
         FROM clusters
         ORDER BY gps_timestamp")
                           .unwrap()
                           .query(&[])
                           .unwrap()
                           .map(|r| r.unwrap())
                           .map(|r| (r.get::<i64>(0),
                                     r.get::<f64>(1),
                                     r.get::<f64>(2),
                                     r.get::<String>(3),
                                     r.get::<i64>(4)))
                           .collect::<Vec<_>>();
    clusters.sort_by(|a, b| a.0.cmp(&b.0));

    let mut query = conn.prepare("SELECT latitude, longitude,
                                         strftime('%Y-%m-%dT%H:%M:%SZ', gps_timestamp, 'unixepoch'),
                                         filename
                                  FROM positions
                                  WHERE cluster_id = ?
                                  ORDER BY gps_timestamp").unwrap();
    println!(r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.0">
	<name>Photos</name>"#);
    println!("<trk><name>Clusters</name><trkseg>");
    for cluster in &clusters {
        if cluster.4 > 5 {
            println!("<trkpt lat=\"{:.8}\" lon=\"{:.8}\"><time>{}</time></trkpt>",
                     cluster.1,
                     cluster.2,
                     cluster.3);
        }
    }
    println!("</trkseg></trk>");

    for cluster in &clusters {
        println!("<trk><name>Photos {}</name><number>{}</number><trkseg>", cluster.0, cluster.0);
        for row in query.query(&[&cluster.0]).unwrap() {
            let row = row.unwrap();
            println!("<trkpt lat=\"{:.8}\" lon=\"{:.8}\"><time>{}</time><name>{}</name></trkpt>",
                     row.get::<f64>(0),
                     row.get::<f64>(1),
                     row.get::<String>(2),
                     row.get::<String>(3));
        }
        println!("</trkseg></trk>");
    }
    println!("</gpx>");
}
