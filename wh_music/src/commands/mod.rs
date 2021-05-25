pub mod join;
pub mod play;
pub mod queue;

use serenity::framework::standard::macros::*;

use join::*;
use play::*;
use queue::*;

#[group]
#[only_in(guild)]
#[commands(join, play, queue)]
pub struct Music;
