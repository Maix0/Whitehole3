#[macro_use]
extern crate sqlx;
#[macro_use]
extern crate fluent_const;
#[macro_use]
extern crate log;
#[macro_use]
extern crate wh_core;
#[macro_use]
extern crate wh_permission;
#[macro_use]
extern crate serde;

extern crate arrayvec;
extern crate aspotify;
extern crate chrono;
extern crate hound;
extern crate once_cell;
extern crate rand;
extern crate reqwest;
extern crate serde_json;
extern crate serenity;
extern crate songbird;
extern crate tokio;
extern crate wh_database;

pub mod commands;
pub mod module;
pub mod shared;
