use rand::{Rng, thread_rng};
use rand::distributions::Uniform;

pub fn roll_die() -> u8 {
    let mut rng = thread_rng();
    let die = Uniform::new_inclusive(1u8, 6u8);
    rng.sample(die)
}
