use thiserror::Error;

#[derive(Error, Debug)]
pub struct ActorDead;

impl std::fmt::Display for ActorDead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActorDead").finish()
    }
}