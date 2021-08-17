#[macro_use]
extern crate log;
extern crate serenity;
#[macro_use]
extern crate sqlx;
#[macro_use]
extern crate wh_core;
extern crate wh_database;
#[macro_use]
extern crate wh_permission;
#[macro_use]
extern crate fluent_const;

mod commands;
mod event_handler;
pub mod module;
pub mod shared;
