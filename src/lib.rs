pub mod client;
pub mod db;
pub mod events;
pub mod game;
#[cfg(feature = "server")]
pub fn random_id() -> String {
    use rand::{distributions::Alphanumeric, Rng};
    /*
    Title: How do I create a random String by sampling from alphanumeric characters?
    Author:  M. Clabaut
    Date: Jan 20 2019
    Availability: https://stackoverflow.com/questions/54275459/how-do-i-create-a-random-string-by-sampling-from-alphanumeric-characters
    */
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect::<String>()
}
