use actix_web::{error, guard, middleware, web, App, Error, HttpResponse, HttpServer};
use crossbeam_channel::Sender;
use karaoke::{
    channel::{WorkerCommand, WORKER_CHANNEL},
    collection::{Collection, Kfile, COLLECTION},
    queue::PLAY_QUEUE,
    CONFIG,
};
use serde_derive::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

#[derive(Deserialize)]
struct Song {
    hash: u64,
}

#[derive(Serialize, Deserialize)]
struct Queue {
    queue: Vec<Kfile>,
}

#[derive(Serialize, Deserialize)]
struct JsonStatus {
    status: &'static str,
    message: Option<String>,
}

fn index(tera: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let context = HashMap::<String, u64>::new();
    let html = tera
        .render("index.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

fn songs(
    tera: web::Data<tera::Tera>,
    collection: web::Data<Collection>,
) -> Result<HttpResponse, Error> {
    let html = tera
        .render("songs.html", collection.get_ref())
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

fn artists(
    tera: web::Data<tera::Tera>,
    collection: web::Data<Collection>,
) -> Result<HttpResponse, Error> {
    let html = tera
        .render("artists.html", collection.get_ref())
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

fn artist(
    tera: web::Data<tera::Tera>,
    hash: web::Path<u64>,
    collection: web::Data<Collection>,
) -> Result<HttpResponse, Error> {
    let artist = collection.by_artist.get(&hash);
    let html = tera
        .render("artist.html", &artist)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

fn queue(
    tera: web::Data<tera::Tera>,
    queue: web::Data<Arc<Mutex<Vec<Kfile>>>>,
) -> Result<HttpResponse, Error> {
    let _queue = queue.lock().unwrap();
    let queue = _queue.clone();
    drop(_queue);

    let queue = Queue { queue };
    let html = tera
        .render("queue.html", &queue)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

fn player(tera: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let context = HashMap::<String, u64>::new();
    let html = tera
        .render("player.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

fn add(
    form: web::Form<Song>,
    collection: web::Data<Collection>,
    worker_sender: web::Data<Sender<WorkerCommand>>,
) -> HttpResponse {
    let hash = form.hash;
    let kfile = collection.by_song[&hash].clone();
    let cmd = WorkerCommand::AddQueue { kfile };
    worker_sender.send(cmd).unwrap();
    HttpResponse::Ok().json(JsonStatus {
        status: "ok",
        message: None,
    })
}

fn playnow(
    form: web::Form<Song>,
    collection: web::Data<Collection>,
    worker_sender: web::Data<Sender<WorkerCommand>>,
) -> HttpResponse {
    let hash = form.hash;
    let kfile = collection.by_song[&hash].clone();
    let cmd = WorkerCommand::PlayNow { kfile };
    worker_sender.send(cmd).unwrap();
    HttpResponse::Ok().json(JsonStatus {
        status: "ok",
        message: None,
    })
}

fn next(worker_sender: web::Data<Sender<WorkerCommand>>) -> HttpResponse {
    let cmd = WorkerCommand::Next;
    worker_sender.send(cmd).unwrap();
    sleep(Duration::from_millis(500));
    HttpResponse::Ok().json(JsonStatus {
        status: "ok",
        message: None,
    })
}

fn clear(worker_sender: web::Data<Sender<WorkerCommand>>) -> HttpResponse {
    let cmd = WorkerCommand::ClearQueue;
    worker_sender.send(cmd).unwrap();
    sleep(Duration::from_millis(500));
    HttpResponse::Ok().json(JsonStatus {
        status: "ok",
        message: None,
    })
}

fn stop(worker_sender: web::Data<Sender<WorkerCommand>>) -> HttpResponse {
    let cmd = WorkerCommand::Stop;
    worker_sender.send(cmd).unwrap();
    sleep(Duration::from_millis(500));
    HttpResponse::Ok().json(JsonStatus {
        status: "ok",
        message: None,
    })
}

fn player_next(queue: web::Data<Arc<Mutex<Vec<Kfile>>>>) -> HttpResponse {
    let _queue = queue.lock().unwrap();
    if _queue.len() == 0 {
        drop(_queue);
        return HttpResponse::Ok().json(JsonStatus {
            status: "ok",
            message: None,
        });
    }

    let next = _queue[0].mp3_path.clone();
    let file_name = next.file_name().unwrap().to_str().unwrap();
    let file_name = file_name.replace(".mp3", "");
    drop(_queue);

    HttpResponse::Ok().json(JsonStatus {
        status: "ok",
        message: Some(file_name),
    })
}

fn p404(tera: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let context = HashMap::<String, u64>::new();
    let html = tera
        .render("404.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::NotFound()
        .content_type("text/html")
        .body(html))
}

fn get_server_port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080)
}

pub fn run() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let port = get_server_port();
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    let server = HttpServer::new(|| {
        let collection = COLLECTION.clone();
        let worker_sender = WORKER_CHANNEL.0.clone();
        let play_queue = PLAY_QUEUE.clone();

        let mut template_path = CONFIG.data_path.clone();
        template_path.push("templates/**/*");
        let tera = tera::Tera::new(template_path.to_str().unwrap()).unwrap();

        let mut static_path = CONFIG.data_path.clone();
        static_path.push("static");
        let mut song_path = CONFIG.data_path.clone();
        song_path.push("songs");

        App::new()
            .data(collection)
            .data(worker_sender)
            .data(play_queue)
            .data(tera)
            .wrap(middleware::Logger::default()) // enable logger
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/songs").route(web::get().to(songs)))
            .service(web::resource("/artists").route(web::get().to(artists)))
            .service(web::resource("/artist/{hash}").route(web::get().to(artist)))
            .service(web::resource("/queue").route(web::get().to(queue)))
            .service(web::resource("/api/add").route(web::post().to(add)))
            .service(web::resource("/api/playnow").route(web::post().to(playnow)))
            .service(web::resource("/api/next").route(web::post().to(next)))
            .service(web::resource("/api/clear").route(web::post().to(clear)))
            .service(web::resource("/api/stop").route(web::post().to(stop)))
            .service(web::resource("/player").route(web::get().to(player)))
            .service(web::resource("/player/next").route(web::get().to(player_next)))
            .service(actix_files::Files::new("/static", static_path))
            .service(actix_files::Files::new("/songs", song_path))
            .default_service(
                // 404 for GET request
                web::resource("")
                    .route(web::get().to(p404))
                    // all requests that are not `GET`
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            )
    })
    .bind(addr)?;

    println!("Actix has launched from http://0.0.0.0:{}", port);

    server.run()
}
