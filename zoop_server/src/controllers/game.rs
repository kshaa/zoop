use crate::actors::player::*;
use crate::actors::room::*;
use crate::domain::error::*;
use crate::domain::lobby::*;
use crate::domain::users::{Ticket, Users};
use actix::*;
use actix_web::web;
use actix_web::web::Data;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::{get, post, Result};
use actix_web_actors::ws;

use std::sync::Mutex;

use zoop_shared::player_id::PlayerId;
use zoop_shared::room_config::GameRoomConfig;
use zoop_shared::room_id::RoomId;

#[post("/game/new/by/{player_id}/ticket/{ticket}/player_count/{player_count}")]
pub async fn game_room_spawn(
    path: web::Path<(PlayerId, Ticket, u32)>,
    users_mutex: Data<Mutex<Users>>,
    lobby_mutex: Data<Mutex<GameLobby>>,
) -> Result<web::Json<RoomId>, AppError> {
    let (player_id, ticket, player_count) = path.as_ref();
    let users = users_mutex.lock().unwrap();
    let is_user_with_ticket = users.has(player_id, ticket.clone());

    if player_count.clone() <= 1 {
        Err(AppError::NotEnoughPlayers())
    } else if !is_user_with_ticket {
        Err(AppError::UserTicketWrong())
    } else {
        let mut lobby = lobby_mutex.lock().unwrap();
        let address = RoomId::new();
        println!("Attempting to create room {}", &address);
        lobby
            .create(address.clone(), player_count.clone(), player_id.clone())
            .map(|_| web::Json(address))
    }
}

#[post("/game/join/{room_id}/by/{player_id}/ticket/{ticket}")]
pub async fn game_room_join(
    path: web::Path<(RoomId, PlayerId, Ticket)>,
    users_mutex: Data<Mutex<Users>>,
    lobby_mutex: Data<Mutex<GameLobby>>,
) -> Result<web::Json<()>, AppError> {
    let (room_id, player_id, ticket) = path.as_ref();
    let users = users_mutex.lock().unwrap();
    let is_user_with_ticket = users.has(player_id, ticket.clone());

    if !is_user_with_ticket {
        Err(AppError::UserTicketWrong())
    } else {
        let mut lobby = lobby_mutex.lock().unwrap();
        let starter = |room_id: RoomId, player_count: u32| {
            GameRoom::of(room_id.clone(), player_count.clone()).start()
        };
        println!("Attempting to enqueue {} in room {}", &player_id, &room_id);
        lobby
            .enqueue_player(room_id.clone(), player_id.clone(), starter)
            .map(|_| web::Json(()))
    }
}

#[get("/game/ready/{room_id}/for/{player_id}/ticket/{ticket}")]
pub async fn game_room_ready(
    path: web::Path<(RoomId, PlayerId, Ticket)>,
    users_mutex: Data<Mutex<Users>>,
    lobby_mutex: Data<Mutex<GameLobby>>,
) -> Result<web::Json<GameRoomConfig>, AppError> {
    let (room_id, player_id, ticket) = path.as_ref();

    let readiness = {
        let users = users_mutex.lock().unwrap();
        let is_user_with_ticket = users.has(player_id, ticket.clone());

        if !is_user_with_ticket {
            Err(AppError::UserTicketWrong())
        } else {
            let mut lobby = lobby_mutex.lock().unwrap();
            lobby.ready(room_id.clone()).clone()
        }
    };

    let lazy_config_result = || {
        let mut lobby = lobby_mutex.lock().unwrap();
        lobby.config(room_id.clone())
    };

    match readiness {
        Ok(None) => lazy_config_result().map(|r| web::Json(r)),
        Ok(Some(notification)) => {
            notification.notified().await;
            lazy_config_result().map(|r| web::Json(r))
        }
        Err(err) => Err(err),
    }
}

#[get("/game/config/{room_id}/for/{player_id}/ticket/{ticket}")]
pub async fn game_room_config(
    path: web::Path<(RoomId, PlayerId, Ticket)>,
    users_mutex: Data<Mutex<Users>>,
    lobby_mutex: Data<Mutex<GameLobby>>,
) -> Result<web::Json<()>, AppError> {
    let (room_id, player_id, ticket) = path.as_ref();
    let users = users_mutex.lock().unwrap();
    let is_user_with_ticket = users.has(player_id, ticket.clone());

    if !is_user_with_ticket {
        Err(AppError::UserTicketWrong())
    } else {
        let mut lobby = lobby_mutex.lock().unwrap();
        lobby.config(room_id.clone()).map(|_| web::Json(()))
    }
}

#[get("/game/connect/{room_id}/as/{player_id}/ticket/{ticket}")]
pub async fn game_room_connect(
    path: web::Path<(RoomId, PlayerId, Ticket)>,
    users_mutex: Data<Mutex<Users>>,
    lobby_mutex: Data<Mutex<GameLobby>>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let (room_id, player_id, ticket) = path.as_ref();
    let users = users_mutex.lock().unwrap();
    let is_user_with_ticket = users.has(player_id, ticket.clone());

    if !is_user_with_ticket {
        Err(actix_web::Error::from(AppError::UserTicketWrong()))
    } else {
        let mut lobby = lobby_mutex.lock().unwrap();
        let room_address = match lobby
            .games
            .get_mut(room_id)
            .map(|m| m.address.get_mut().clone()) {
                Some(address) => address,
                None => None
            };
        match room_address {
            Some(address) => {
                let actor = GamePlayer {
                    room_id: room_id.to_owned(),
                    player_id: player_id.clone(),
                    room_address: address.clone(),
                };
                ws::start(actor, &req, stream)
            }
            None => Err(actix_web::Error::from(AppError::GameDoesNotExist {
                id: room_id.clone(),
            })),
        }
    }
}
