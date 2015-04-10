extern crate iron;
extern crate router;
extern crate env_logger;
extern crate num_cpus;
extern crate flate2;
extern crate rustc_serialize;
extern crate rusqlite;

#[macro_use]
extern crate log;

use flate2::FlateWriteExt;

use iron::{status, mime, headers};
use iron::headers::Encoding;
use iron::prelude::*;
use router::Router;

use std::collections::{HashMap, VecDeque};
use std::env;
use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::process;
use std::sync::{Arc, RwLock, Mutex};

fn main() {
    let threads = env::var("WEB_THREADS").map(|s| s.parse().unwrap()).ok()
        .unwrap_or(num_cpus::get() * 6);

    env_logger::init().unwrap();
    match main_(threads) {
        Ok(()) => {}
        Err(e) => {
            write!(&mut io::stderr(), "an error occurred: {}", e).unwrap();
            process::exit(1);
        }
    }
}

fn serve_concat(r: &mut Router, path: &str,
                mime: &str,
                template: Option<&HashMap<String, String>>,
                files: &[&str]) -> io::Result<()> {
    let mut data = String::new();

    for file in files {
        try!(try!(File::open(file)).read_to_string(&mut data));
        data.push_str("\n");
    }

    if let Some(tmpl_data) = template {
        for (k, v) in tmpl_data {
            data = data.replace(&format!("{}{}{}", "{{", k, "}}"),
                                v)
        }
    }

    let mut gzipper = vec![].gz_encode(flate2::Compression::Best);
    gzipper.write_all(data.as_bytes()).unwrap();
    let gzip = gzipper.finish().unwrap();

    let mut deflater = vec![].deflate_encode(flate2::Compression::Best);
    deflater.write_all(data.as_bytes()).unwrap();
    let deflate = deflater.finish().unwrap();

    let gzip_preferred = gzip.len() < deflate.len();

    info!("registering {}, {:?}, size: raw {}, gzip {}, deflate {}", path, files,
             data.len(), gzip.len(), deflate.len());



    let mime: mime::Mime = mime.parse().unwrap();

    r.get(path, move |req: &mut Request| -> IronResult<Response> {
        info!("serving: {}", req.url);
        let mut resp = Response::with(mime.clone());

        let mut accepts_gzip = false;
        let mut accepts_deflate = false;

        if let Some(h) = req.headers.get::<headers::AcceptEncoding>() {
            for e in &h.0 {
                match e.item {
                    Encoding::Gzip => accepts_gzip = true,
                    Encoding::Deflate => accepts_deflate = true,
                    _ => {}
                }
            }
        }
        let use_gzip = accepts_gzip && (!accepts_deflate || gzip_preferred);
        let use_deflate = accepts_deflate && !use_gzip;

        let (encoding, data) = if use_gzip {
            (Some(Encoding::Gzip), &*gzip)
        } else if use_deflate {
            (Some(Encoding::Deflate), &*deflate)
        } else {
            (None, data.as_bytes())
        };
        if let Some(enc) = encoding {
            resp.headers.set(headers::ContentEncoding(vec![enc]));
        }
        resp.set_mut((status::Ok, data));

        Ok(resp)
    });
    Ok(())
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
struct Point {
    lat: f64,
    lon: f64,
    timestamp: u64,
    hdop: f64,
    altitude: f64,
    speed: f64,
}
fn live(router: &mut Router) -> io::Result<()> {
    const HISTORY: usize = 100;

    let conn = rusqlite::SqliteConnection::open(std::path::Path::new("live.sqlite3")).unwrap();



    let mut points = {
        let mut query = conn.prepare("SELECT latitude, longitude, gps_timestamp, hdop, \
                                      altitude, speed \
                                      FROM positions \
                                      ORDER BY gps_timestamp DESC LIMIT ?").unwrap();
        query.query(&[&(HISTORY as i64)]).unwrap().map(|r| r.unwrap())
            .map(|r| {
                Point {
                    lat: r.get(0),
                    lon: r.get(1),
                    timestamp: r.get::<i64>(2) as u64,
                    hdop: r.get(3),
                    altitude: r.get(4),
                    speed: r.get(5)
                }
            }).collect::<VecDeque<Point>>()
    };
    {
        // reverse the points, so they're arranged from oldest to newest
        let mut iter = points.iter_mut();
        while let (Some(a), Some(b)) = (iter.next(), iter.next_back()) {
            std::mem::swap(a, b)
        }
    }
    info!("loaded {} points", points.len());
    let points = Arc::new(RwLock::new(points));
    let points_ = points.clone();

    let mut html = String::new();
    try!(try!(File::open("web/live.html")).read_to_string(&mut html));
    let mut html = html.split("{{points}}");
    let html1 = html.next().unwrap().to_string();
    let html2 = html.next().unwrap().to_string();

    let html_mime = "text/html; charset=utf-8".parse::<mime::Mime>().unwrap();
    router.get("/live", move |_req: &mut Request| -> IronResult<Response> {
        let resp = format!("{}{}{}",
                           html1,
                           rustc_serialize::json::as_json(&*points_.read().unwrap()),
                           html2);
        Ok(Response::with((status::Ok, resp)).set(html_mime.clone()))
    });
    try!(serve_concat(router, "/live/style.css", "text/css", None,
                      &[
                          "web/js/vendored/Leaflet-MiniMap/Control.MiniMap.min.css",
                          "web/css/minimap-override.css",
                          "web/css/live-style.css",
                          ]));
    try!(serve_concat(router, "/live/script.js", "application/js", None,
                      &[
                          "web/js/vendored/leaflet-hash.js",
                          "web/js/vendored/Leaflet-MiniMap/Control.MiniMap.min.js",
                          "web/js/live-script.js",
                          ]));


    let points_ = points.clone();
    router.get("/live/points.json", move |req: &mut Request| -> IronResult<Response> {
        let points = points_.read().unwrap();
        let since = match req.url.query {
            Some(ref query) => match query.parse() {
                Ok(since) => since,
                Err(e) => return Ok(Response::with((status::BadRequest,
                                                    format!("invalid `since`: {}", e))))
            },
            None => 0,
        };
        let points = points.iter().filter(|p: &&Point| p.timestamp > since).collect::<Vec<_>>();
        Ok(Response::with((status::Ok,
                           rustc_serialize::json::as_json(&points).to_string())))
    });

    let conn = Mutex::new(conn);
    let route = "/live/register/:lat/:lon/:timestamp/:hdop/:altitude/:speed";
    router.get(route, move |req: &mut Request| -> IronResult<Response> {
        println!("url = {}", req.url);
        macro_rules! make {
            ($($name: ident),*) => {
                Point {
                    $($name: match req.extensions.get::<Router>().unwrap()
                      .find(stringify!($name)).unwrap().parse() {
                          Ok(x) => x,
                          Err(e) => {
                              return Ok(Response::with((status::BadRequest,
                                                        format!("invalid value for {}: {}",
                                                                stringify!($name),
                                                                e))))
                          }
                      },)*
                }
            }
        }
        let mut point = make!(lat, lon, timestamp, altitude, speed, hdop);
        let local_time = point.timestamp;
        point.timestamp -= 10 * 60 * 60 * 1000;
        conn.lock().unwrap().execute(
            "INSERT INTO positions (true_latitude, true_longitude, near_home, \
             latitude, longitude, local_timestamp, gps_timestamp, altitude, \
             speed, hdop) VALUES (?,?,?,?,?,?,?,?,?,?)",
            &[&point.lat, &point.lon, &0, &point.lat, &point.lon,
              &(local_time as i64), &(point.timestamp as i64),
              &point.altitude, &point.speed, &point.hdop]).unwrap();

        let mut points = points.write().unwrap();
        points.push_back(point);
        if points.len() > HISTORY {
            points.pop_front();
        }
        Ok(Response::with((status::Ok, "registered")))
    });
    Ok(())
}

fn main_(threads: usize) -> io::Result<()> {
    let mut router = Router::new();

    let mut summary = String::new();
    try!(try!(File::open("web/summary.json")).read_to_string(&mut summary));

    try!(serve_concat(&mut router, "/", "text/html; charset=utf-8",
                      Some(&Some(("summary".to_string(), summary)).into_iter().collect()),
                      &["web/index.html"]));
    try!(serve_concat(&mut router, "/style.css", "text/css", None,
                      &[
                          "web/js/vendored/Leaflet-MiniMap/Control.MiniMap.min.css",
                          "web/css/minimap-override.css",
                          "web/css/style.css",
                          ]));
    try!(serve_concat(&mut router, "/script.js", "application/js", None,
                      &[
                          "web/js/vendored/leaflet-hash.js",
                          "web/js/vendored/Leaflet-MiniMap/Control.MiniMap.min.js",
                          "web/js/script.js",
                          ]));

    try!(serve_concat(&mut router, "/data/summary.json", "application/json", None,
                      &["web/summary.json"]));
    try!(serve_concat(&mut router, "/data/clusters.json", "application/json", None,
                      &["web/clusters.json"]));
    try!(live(&mut router));

    let bind = "0.0.0.0:4444";
    println!("listening on {}", bind);
    Iron::new(router)
        .listen_with(bind, threads, iron::Protocol::Http)
        .unwrap();
    Ok(())
}
