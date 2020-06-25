use actix_web::{
    error, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, // http::StatusCode,
};
use bytes::{// Bytes,
            BytesMut};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

const MAX_SIZE: usize = 262_144;

use structopt::StructOpt;

use chrono::prelude::*;

use csi_types::{ser::SerCSI, ser::abs
                // CSIStruct, CSI, ComplexDef
};

mod types;
use types::*;

mod common;
use common::*;

use std::sync::Mutex;
use std::fs::File;

#[derive(Debug, StructOpt)]
#[structopt(name = "recv_csi_server", about = "Receive CSI data Server")]
struct Opt {
    /// Activate debug mode
    // short and long flags (-d, --debug) will be deduced from the field's name
    #[structopt(short, long)]
    debug: bool,

    #[structopt(long)]
    addr: String,

    #[structopt(long)]
    write_at_least: Option<usize>,

    #[structopt(long, short = "p")]
    is_present: bool,

    #[structopt(long, short)]
    x: Option<f64>,

    #[structopt(long, short)]
    y: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct XYData {
    x: f64,
    y: f64,
}

async fn post_csi(mut payload: web::Payload, shared_state: web::Data<Mutex<CSIData>>) -> Result<HttpResponse, Error> {
    let mut body = BytesMut::new();
    let x = &mut *shared_state.lock().unwrap();
    
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    let body: SerCSI = bincode::deserialize(&body)
        .unwrap();

    let m = body.csi_matrix.clone();
    let mm: Vec<Vec<Vec<f64>>> = m.iter().map(
        |a| a.iter().map(
            |b| b.iter().map(
                |x| abs(x.clone())
            )
            // there are almost always 56 channels in CSI data
                .take(56)
                .collect()
        ).collect()
    ).collect();

    let recent_xy = x.recent_xy;

    let sample = Sample {
        date: Utc::now(),
        x: recent_xy.0,
        y: recent_xy.1,
        csi: mm.clone(),
    };

    // *shared_state.lock().unwrap() =
    // *x = CSIData { inner: mm.clone() };
    (*x).inner.push(mm.clone());
    (*x).samples.push(sample.clone());


    if let Some(cfg) = (*x).c.clone() {
        if x.samples.len() == cfg.write_at_least {
            save_collected(&x.samples);

            println!("Done saving");
        }
    }

    Ok(HttpResponse::Ok().body("")) // <- send response
}

/// Update the most recent position
async fn post_xy(mut payload: web::Payload, shared_state: web::Data<Mutex<CSIData>>) -> Result<HttpResponse, Error> {
    let mut body = BytesMut::new();
    let d = &mut *shared_state.lock().unwrap();

    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    let body: XYData = bincode::deserialize(&body)
        .unwrap();

    (*d).recent_xy = (body.x, body.y);


    Ok(HttpResponse::Ok().body("")) // <- send response
}

async fn index(_req: HttpRequest, shared_state: web::Data<Mutex<CSIData>>) -> Result<HttpResponse, Error> {
    let x = &*shared_state.lock().unwrap();
    // let data = data.data;
    
    // shared
    Ok(HttpResponse::Ok().json(x.inner.clone()))
}

async fn get_one(_req: HttpRequest, shared_state: web::Data<Mutex<CSIData>>) -> Result<HttpResponse, Error> {
    let x = &*shared_state.lock().unwrap();
    // let data = data.data;
    
    // shared
    Ok(HttpResponse::Ok().json(vec![x.inner.last().clone()]))
}


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    env_logger::init();

    let shared_data = web::Data::new(Mutex::new(
        CSIData {
            inner: vec![] ,
            
            c: if opt.write_at_least.is_some() {
                Some(WriteConfig {
                    write_at_least: opt.write_at_least.unwrap(),
                    data: Receive::Realtime,
                })
            } else {
                None
            },

            recent_xy: (-1.0, -1.0),

            samples: vec![],
        }
    ));

    HttpServer::new(move || {
        App::new()
            .app_data(shared_data.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/csi").route(web::post().to(post_csi)))
            .service(web::resource("/post_xy").route(web::post().to(post_xy)))
            .service(web::resource("/get").to(index))
            .service(web::resource("/get_one").to(get_one))
    })
        .bind(opt.addr)?
        .run()
        .await
}
