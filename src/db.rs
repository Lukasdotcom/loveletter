use cfg_if::cfg_if;
cfg_if! {
if #[cfg(feature = "server")] {
    use sqlx::Connection;
    #[derive(sqlx::FromRow, Debug)]
    pub struct Games {
        pub id: String,
        pub turn: i32,
        pub deck: String,
        pub players: i32,
        pub played_card: i32,
        pub player_pick: i32,
    }
    #[derive(sqlx::FromRow, Debug)]
    pub struct Players {
        pub id: String,
        pub turn : i32,
        pub game: String,
        pub name: String,
        pub hand: String,
        pub immune: bool,
        pub alive: bool,
        pub text: String,
        pub input: bool,
    }
    use sqlx::{SqliteConnection, Error};
    pub async fn db() -> Result<SqliteConnection, Error> {
        SqliteConnection::connect("sqlite:./db/db.db")
            .await
        }
    }
}
