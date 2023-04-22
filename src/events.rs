use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventData {
    pub text: String,
    pub input: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub game: String,
    pub player: Option<i32>,
    pub event: EventData,
}
cfg_if! {
  if #[cfg(feature = "server")] {
      use broadcaster::BroadcastChannel;
      lazy_static::lazy_static! {
        pub static ref GAME_EVENTS: BroadcastChannel<Event> = BroadcastChannel::new();
      }
      pub async fn store_events() {
          use crate::db::*;
          let mut conn = db().await.expect("couldn't connect to DB");
          let mut events = GAME_EVENTS.clone();
          while let Some(event) = events.recv().await {
            println!("Event fired: {:?}", event);
            match event.player {
              Some(player) => {
                sqlx::query("UPDATE Players SET text=text || ?, input=? WHERE game=? AND turn=?")
                  .bind(event.event.text)
                  .bind(event.event.input)
                  .bind(&event.game)
                  .bind(player)
                  .execute(&mut conn)
                  .await
                  .expect("couldn't insert event");
              },
              None => {
                sqlx::query("UPDATE Players SET text=text || ?, input=? WHERE game=?")
                  .bind(event.event.text)
                  .bind(event.event.input)
                  .bind(&event.game)
                  .execute(&mut conn)
                  .await
                  .expect("couldn't insert event");
            }
          }
      }
    }
  }
}
