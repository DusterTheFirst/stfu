#![allow(unused)]

use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Action {
    Mute(u64),
    Unmute(u64),
}
impl Action {
    pub fn max_size() -> usize {
        const SAMPLE_ACTION: Action = Action::Mute(0);
        bincode::serialized_size(&SAMPLE_ACTION)
            .unwrap()
            .try_into()
            .unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Acknowledgement {
    Success(Action),
    Failure,
}
impl Acknowledgement {
    pub fn max_size() -> usize {
        const SAMPLE_ACK: Acknowledgement = Acknowledgement::Success(Action::Mute(0));
        bincode::serialized_size(&SAMPLE_ACK)
            .unwrap()
            .try_into()
            .unwrap()
    }
}
