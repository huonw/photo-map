extern crate hyper;
extern crate nickel;
#[macro_use] extern crate nickel_macros;
extern crate env_logger;

use hyper::net;
use hyper::method::Method;
use nickel::{Nickel, Request, Response, Router, Middleware, MiddlewareResult, HttpRouter};
use nickel::mimes::MediaType;

use std::collections::HashMap;
use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::process;

fn main() {
    env_logger::init().unwrap();
    match main_() {
        Ok(()) => {}
        Err(e) => {
            write!(&mut io::stderr(), "an error occurred: {}", e).unwrap();
            process::exit(1);
        }
    }
}

fn serve_concat(r: &mut Router, path: &str,
                mime: MediaType,
                template: Option<&HashMap<String, String>>,
                files: &[&str]) -> io::Result<()> {

    struct Data {
        mime: MediaType,
        data: String
    }
    impl Middleware for Data {
        fn invoke<'a, 'b>(&'a self,
                          req: &mut Request<'b, 'a, 'b>,
                          mut res: Response<'a, net::Fresh>)
                          -> MiddlewareResult<'a>
        {
            println!("serving: {:?}", req.origin.uri);
            res.content_type(self.mime);
            res.send(&*self.data)
        }
    }
    let mut data = String::new();

    for file in files {
        try!(try!(File::open(file)).read_to_string(&mut data));
        data.push_str("\n");
    }

    println!("registering {}, {:?}, {} bytes", path, files, data.len());

    if let Some(tmpl_data) = template {
        for (k, v) in tmpl_data {
            data = data.replace(&format!("{}{}{}", "{{", k, "}}"),
                                v)
        }
    }

    r.get(path, Data { mime: mime, data: data });
    Ok(())
}

fn main_() -> io::Result<()> {
    let mut server = Nickel::new();

    // middleware is optional and can be registered with `utilize`
    server.utilize(middleware! { |request|
        println!("logging request: {:?}", request.origin.uri);
    });
    let mut router = Nickel::router();

    let mut summary = String::new();
    try!(try!(File::open("web/summary.json")).read_to_string(&mut summary));

    try!(serve_concat(&mut router, "/", MediaType::Html,
                      Some(&Some(("summary".to_string(), summary)).into_iter().collect()),
                      &["web/index.html"]));
    try!(serve_concat(&mut router, "/style.css", MediaType::Css, None,
                      &[
                          "web/js/vendored/Leaflet-MiniMap/Control.MiniMap.min.css",
                          "web/css/minimap-override.css",
                          "web/css/style.css",
                          ]));
    try!(serve_concat(&mut router, "/script.js", MediaType::Js, None,
                      &[
                          "web/js/vendored/leaflet-hash.js",
                          "web/js/vendored/Leaflet-MiniMap/Control.MiniMap.min.js",
                          "web/js/script.js",
                          ]));

    try!(serve_concat(&mut router, "/data/summary.json", MediaType::Json, None,
                      &["web/summary.json"]));
    try!(serve_concat(&mut router, "/data/clusters.json", MediaType::Json, None,
                      &["web/clusters.json"]));

    server.utilize(router);

    server.listen("0.0.0.0:4444");

    Ok(())
}
