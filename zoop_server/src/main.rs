mod actors;
mod controllers;
mod domain;

use actix_cors::Cors;
use actix_files::Files;
use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
use std::sync::Mutex;

use crate::controllers::game::*;
use crate::controllers::user::*;
use crate::domain::lobby::*;
use crate::domain::users::Users;
use crate::domain::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let lobby = Data::new(Mutex::new(GameLobby::default()));
    let users = Data::new(Mutex::new(Users::default()));

    let host = "127.0.0.1";
    let port = 8080;
    println!("Starting game server on http://{}:{}", host, port);
    HttpServer::new(move || {
        // TODO: For dev, I want cross-domain requests, for prod - actually read docs
        let cors = Cors::permissive();
        App::new()
            .app_data(lobby.clone())
            .app_data(users.clone())
            .service(game_room_spawn)
            .service(game_room_join)
            .service(game_room_ready)
            .service(game_room_connect)
            .service(game_room_config)
            .service(user_create)
            .service(Files::new("/static", "./static"))
            .wrap(cors)
    })
    .bind((host, port))?
    .run()
    .await
}
