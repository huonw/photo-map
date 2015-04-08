extern crate rexiv2;
extern crate getopts;
extern crate simple_parallel;
extern crate rusqlite;
extern crate num;
extern crate chrono;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rand;
extern crate num_cpus;

use rand::Rng;

use std::collections::HashSet;
use std::sync::Mutex;
use std::{env, io, process};
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;


fn main() {
    env_logger::init().unwrap();

    let mut args = env::args();
    let program = args.next().unwrap();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this message");
    opts.optopt("t", "threads", "number of threads to use", "NUM");
    opts.reqopt("d", "database", "SQLite database to insert records into", "FILE");
    opts.optopt("f", "files-from",
                "file containing (extra) file names to read from, one per line",
                "FILE");
    opts.optopt("c", "home", "home coords, to randomise values around", "NUM,NUM");

    let matches = match opts.parse(args) {
        Ok(m) => m,
        Err(e) => {
            writeln!(&mut io::stderr(), "invalid arguments: {}", e).unwrap();
            process::exit(1);
        }
    };

    if matches.opt_present("h") {
        println!("{}", opts.usage(&format!("Usage: {} [options]", program)));
        return
    }
    let nthreads = match matches.opt_str("t").map(|s| s.parse()) {
        None => num_cpus::get(),
        Some(Ok(t)) => t,
        Some(Err(e)) => {
            panic!("invalid argument -t: {}", e)
        }
    };

    let files_from = match matches.opt_str("f") {
        Some(s) => match files_from(Path::new(&s)) {
            Ok(ff) => ff,
            Err(e) => {
                panic!("invalid files-from: {}", e)
            }
        },
        None => vec![]
    };

    let (home_lat, home_lon) = match matches.opt_str("c") {
        None => (std::f64::INFINITY, std::f64::INFINITY),
        Some(s) => {
            let mut parts = s.split(',');
            let lon = parts.next().expect("invalid argument -c").trim();
            let lat = parts.next().expect("invalid argument -c").trim();

            match lon.parse().and_then(|lon| lat.parse().map(|lat| (lat, lon))) {
                Ok(t) => t,
                Err(e) => panic!("invalid argument -c: {}", e),
            }
        }
    };

    let db_file = &matches.opt_str("d").unwrap();
    let db_file = Path::new(db_file);
    let conn = rusqlite::SqliteConnection::open_with_flags(db_file,
                                                           rusqlite::SQLITE_OPEN_READ_WRITE)
        .unwrap();
    let m_conn = Mutex::new(conn);

    let mut pool = simple_parallel::Pool::new(nthreads);
    let total = matches.free.len() + files_from.len();
    let mut iter = matches.free.into_iter().chain(files_from.into_iter()).enumerate();
    if let Some((i, file)) = iter.next() {
        // "initialise" gexiv by running it on one thread to start with.
        handle_file(i + 1, total, file, &m_conn, home_lat, home_lon);
    }
    pool.for_(iter,
              |(i, file)| handle_file(i + 1, total, file, &m_conn, home_lat, home_lon));
}

fn files_from(file: &Path) -> io::Result<Vec<String>> {
    debug!("reading files-from: {:?}", file);
    let mut file = try!(File::open(file));
    let mut s = String::new();
    try!(file.read_to_string(&mut s));
    Ok(s.lines().map(|s| s.trim_right_matches('\n').to_string()).collect())
}

fn handle_file(idx: usize, total: usize,
               file: String, conn: &Mutex<rusqlite::SqliteConnection>,
               home_lat: f64, home_lon: f64) {
    debug!("{:5}/{}: {}: starting", idx, total, file);
    let exif = match rexiv2::Metadata::new_from_path(&file) {
        Ok(exif) => exif,
        Err(e) => {
            writeln!(&mut io::stderr(),
                     "error reading EXIF from {}: {}", file, e).unwrap();
            return
        }
    };

    let gps = match exif.get_gps_info() {
        Some(g) => g,
        None => {
            info!("{:5}/{}: {}: no gps info", idx, total, file);
            return
        }
    };
    let (near_home, userfacing_lat, userfacing_lon)
        = tweak_gps_if_near_home(gps.latitude, gps.longitude, home_lat, home_lon);

    let tags = exif.get_exif_tags().unwrap().into_iter().collect::<HashSet<_>>();

    let camera_datetime = if tags.contains("Exif.Image.DateTime") {
        exif.get_tag_string("Exif.Image.DateTime").unwrap()
    } else if tags.contains("Exif.Photo.DateTimeOriginal") {
        exif.get_tag_string("Exif.Photo.DateTimeOriginal").unwrap()
    } else {
        info!("{:5}/{}: {}: no usable time data", idx, total, file);
        return
    };
    let camera_datetime = chrono::NaiveDateTime::parse_from_str(&camera_datetime,
                                                                "%Y:%m:%d %H:%M:%S").unwrap();

    let gps_date = exif.get_tag_string("Exif.GPSInfo.GPSDateStamp").unwrap();
    let gps_date = chrono::NaiveDate::parse_from_str(&gps_date, "%Y:%m:%d").unwrap();

    let gps_time = exif.get_tag_string("Exif.GPSInfo.GPSTimeStamp").unwrap();
    let mut gps_time_parts = gps_time.split(' ');
    let hours = parse_next_integer_ratio(&mut gps_time_parts);
    let minutes = parse_next_integer_ratio(&mut gps_time_parts);
    let seconds = parse_next_integer_ratio(&mut gps_time_parts);
    let gps_datetime = gps_date.and_hms(hours, minutes, seconds);

    let camera_timestamp = camera_datetime.timestamp();
    let gps_timestamp = gps_datetime.timestamp();
    debug!("{:5}/{}: {}: camera = {}, gps = {}, {:?}",
           idx, total,
           file, camera_datetime, gps_datetime, gps);

    let conn = conn.lock().unwrap();

    let mut already_exists = false;
    let _ = conn.query_row_safe("SELECT 1 FROM positions
                                 WHERE true_latitude = ? AND true_longitude = ?
                                   AND gps_timestamp = ? AND filename = ?",
                                &[&gps.latitude, &gps.longitude, &gps_timestamp, &file],
                                |_| already_exists = true);
    if already_exists {
        // already processed
        info!("{:5}/{}: {}: already processed", idx, total, file);
        return;
    }

    conn.execute("INSERT INTO positions (true_latitude, true_longitude,
                                         near_home, latitude, longitude,
                                         camera_timestamp, gps_timestamp, filename)
                  VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                 &[&gps.latitude, &gps.longitude,
                   &(near_home as i64), &userfacing_lat, &userfacing_lon,
                   &camera_timestamp, &gps_timestamp, &file])
        .unwrap();
    info!("{:5}/{}: {}: inserted ok", idx, total, file);
}

fn parse_next_integer_ratio<'a, It: Iterator<Item = &'a str>>(it: &mut It) -> u32 {
    let part = it.next().unwrap().parse::<num::Rational>().unwrap();
    part.to_integer() as u32
}

fn round_to_multiple(x: f64, m: f64) -> f64 {
    (x / m).round() * m
}

const LAT_EPS: f64 = 0.06;
const LON_EPS: f64 = 0.06;
fn tweak_gps_if_near_home(lat: f64, lon: f64, home_lat: f64, home_lon: f64) -> (bool, f64, f64) {
    let top = round_to_multiple(home_lat + LAT_EPS, LAT_EPS);
    let bottom = round_to_multiple(home_lat - LAT_EPS, LAT_EPS);
    let left = round_to_multiple(home_lon - LON_EPS, LON_EPS);
    let right = round_to_multiple(home_lon + LON_EPS, LON_EPS);

    if left <= lon && lon <= right &&
        bottom <= lat && lat <= top
    {
        let mut rng = rand::thread_rng();
        (true,
         rng.gen_range(bottom, top),
         rng.gen_range(left, right))
    } else {
        (false, lat, lon)
    }

}
