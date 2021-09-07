#[macro_use]
extern crate log;
#[macro_use]
extern crate sqlx;
#[macro_use]
extern crate wh_core;
#[macro_use]
extern crate wh_permission;
#[macro_use]
extern crate fluent_const;
extern crate dotenv;
extern crate image;
extern crate once_cell;
extern crate reqwest;
extern crate serde;
extern crate serenity;
extern crate wh_config;
extern crate wh_database;

mod commands;
mod event_handler;
pub mod module;
pub mod shared;
