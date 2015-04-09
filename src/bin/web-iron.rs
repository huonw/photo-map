extern crate iron;
extern crate router;
extern crate env_logger;
extern crate num_cpus;
extern crate flate2;

#[macro_use]
extern crate log;

use flate2::FlateWriteExt;

use iron::{status, mime, headers};
use iron::headers::Encoding;
use iron::prelude::*;
use router::Router;

use std::collections::HashMap;
use std::env;
use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::process;

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

    let bind = "0.0.0.0:4444";
    println!("listening on {}", bind);
    Iron::new(router)
        .listen_with(bind, threads, iron::Protocol::Http)
        .unwrap();
    Ok(())
}
