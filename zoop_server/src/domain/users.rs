use crate::error::*;
use actix_web::Result;
use passwords::PasswordGenerator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zoop_shared::player_id::PlayerId;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ticket(String);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: PlayerId,
    pub username: String,
    pub ticket: Ticket,
}
impl User {
    fn generate(username: String) -> User {
        let id = PlayerId::new();
        let pg = PasswordGenerator {
            length: 20,
            numbers: true,
            lowercase_letters: true,
            uppercase_letters: true,
            symbols: false,
            spaces: false,
            exclude_similar_characters: false,
            strict: true,
        };
        let ticket = Ticket(pg.generate_one().unwrap());
        User {
            id: id.clone(),
            username,
            ticket,
        }
    }
}

#[derive(Default)]
pub struct Users {
    pub users: HashMap<PlayerId, User>,
    pub user_names: HashMap<String, PlayerId>,
}
impl Users {
    pub fn add(&mut self, username: String) -> Result<User, AppError> {
        let existing_user = self.user_names.get(username.clone().trim());
        let new_user = match existing_user {
            None => Ok(User::generate(username.clone())),
            Some(_) => Err(AppError::UsernameAlreadyExists {
                username: username.clone(),
            }),
        };
        let alphanumeric_user = if username
            .clone()
            .chars()
            .all(|c| char::is_alphanumeric(c) || char::is_whitespace(c))
        {
            new_user
        } else {
            Err(AppError::NotAlphanumericUsername {
                username: username.clone(),
            })
        };
        let max_length = 30;
        let short_user = if username.len() <= max_length {
            alphanumeric_user
        } else {
            Err(AppError::TooLongUsername {
                max_length: max_length as u32,
                username: username.clone(),
            })
        };
        let registered_user = match (short_user
            .map(|user| {
                self.user_names.insert(username.clone(), user.id.clone());
                match self.users.insert(user.id.clone(), user.clone()) {
                    None => Ok(user),
                    Some(_) => Err(AppError::UserAlreadyExists {
                        id: user.id.clone(),
                    }),
                }
            })) {
                Ok(user) => user,
                Err(e) => Err(e)
            };

        registered_user
    }
    pub fn has(&self, player_id: &PlayerId, ticket: Ticket) -> bool {
        self.users
            .get(player_id)
            .filter(|u| u.ticket == ticket)
            .is_some()
    }
}
