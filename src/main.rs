extern crate actix;
extern crate actix_web;

#[macro_use]
extern crate askama;

use askama::Template;

use actix::prelude::*;
use actix_web::{
    http, middleware, server, App, HttpResponse, HttpRequest, Path,
    State,
};

use std::collections::BTreeMap;

#[derive(Template)]
#[template(path = "list.html")]
struct ListTemplate<'a> {
    name: &'a str,
    list: &'a Vec<&'a str>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    list: Vec<&'a str>,
}

struct AppState<'a> {
    // Maintain a map of all the lists and their items.
    db: BTreeMap<&'a str, Vec<&'a str>>,
}

impl<'a> AppState<'a> {
    fn new() -> Self {
        let mut s = AppState{
            db: BTreeMap::new()
        };

        // Now build our lists

        s.db.insert("poleshow", vec![
            "IBIS off",
            "50-140 OIS on",
            "Aperture to f/3.6 or higher",
            "ISO to 6400",
            "Shutter to 1/250 or faster (use mode T)",
            "White Balance based on stage neutral (grey card)",
            "White Balance alt - incandesent or fluro 2",
            "AF to C",
            "AF track to 6",
            "AF to centre points (phase)",
            "AF set in both orientations (X-H1)",
            "Focus center or above centre",
            "UV filter",
            "Land scape often better than portrait",
            "Synchronise clocks",
            "Use body custom 2 (pole) (NR + Sharp)",
            "Shoot wide",
            "Shoot angle from low",
        ]);

        s.db.insert("outdoor flash", vec![
            "IBIS off",
            "WB to daylight/shade",
            "WB based on grey card",
            "Shutter based on flash sync (180 or 250)",
            "Control BG light no flash, then add flash",
            "Only ISO/F control flash, not shutter",
            "Use flash manual (not TTL)",
            "UV filter or CPL filter",
            "Look for potential reflections",
        ]);

        s.db.insert("outdoor reflector", vec![
            "IBIS off",
            "White reflector bright sun/direct",
            "Silver/Gold for shade or spotlight",
            "WB to ambient no reflector",
            "CPL or UV filter",
        ]);

        s.db.insert("outdoor pole", vec![
            "IBIS off",
            // from outdoor flash. Can we just copy the vec?
            "WB to daylight/shade",
            "WB based on grey card",
            "Shutter based on flash sync (180 or 250)",
            "Control BG light no flash, then add flash",
            "Only ISO/F control flash, not shutter",
            "Use flash manual (not TTL)",
            "UV filter or CPL filter",
            "Look for potential reflections",
            // pole specific
            "clear BG behind pole (simple-exlusion)",
            "Avoid floor (low angle)",
            "50mm or higher length",
            "shoot wide full BG + pole",
            "Flash high or low to sides (fill or spotlight)",
            "Flash opposite side to sun",
            "Reflector could be used",
        ]);

        s.db.insert("food restaurant", vec![
            "WB is critically important",
            "F2.5 or greater",
            "IBIS on",
            "ISO auto 2 (6400)",
            "Low and High angles (never eye level)",
            "Rotate plates multiple times for possible angles",
            "Swirl wine to light sources",
            "Use available reflections to create effects (IE bottles)",
            "Low angle room shots",
        ]);

        // return our state now
        s
    }
}

/// Async request handler
fn list_view(
    (listname, state): (Path<String>, State<AppState>),
) -> HttpResponse {
    let lns = listname.into_inner();

    state.db
        .get::<str>(&lns)
        .map_or(
            // default
            HttpResponse::NotFound()
                .content_type("text/html")
                .body("Checklist Not Found"),
            // if some
            |l| {
            let s = ListTemplate {
                    name: lns.as_ref(),
                    list: l,
                }.render()
                 .unwrap();
            HttpResponse::Ok()
                .content_type("text/html")
                .body(s)
            }
        )
}

fn index_view(req: &HttpRequest<AppState>) -> HttpResponse {

    let l: Vec<_> = req.state().db.keys().cloned().collect();

    let s = IndexTemplate {
            list: l,
        }.render()
         .unwrap();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(s)
}

fn main() {
    let sys = actix::System::new("checklists");

    // Start http server
    server::new(move || {
        App::with_state(AppState::new())
            // For production
            .prefix("/list")
            // enable logger
            .middleware(middleware::Logger::default())
            // lists
            .resource("/{listname}", |r| r.method(http::Method::GET).with(list_view))
            // index
            .resource("", |r| r.f(index_view))
            .resource("/", |r| r.f(index_view))
    }).bind("127.0.0.1:8080")
        .unwrap()
        .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
