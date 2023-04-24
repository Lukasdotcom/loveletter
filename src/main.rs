use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature = "server")] {
        use futures::StreamExt;
        use serde::Deserialize;
        use loveletter::{db, game, events::{Event, EventData, GAME_EVENTS}};
        use actix_web::*;
        use actix_files::*;
        #[get("{filename:.*}")]
        async fn static_files(req: HttpRequest) -> impl Responder {
            match req.path() {
                "/styles.css" => NamedFile::open("./static/styles.css"),
                "/" => NamedFile::open("./static/index.html"),
                _ => NamedFile::open("./static/404.html"),
            }
        }
        #[get("index.js")]
        async fn javascript() -> impl Responder {
            use std::io::Read;
            use std::env;
            let domain = env::var("ANALYTICS");
            let extra_text = match domain {
                Ok(domain) => format!("var _paq = window._paq = window._paq || [];
_paq.push(['trackPageView']);
_paq.push(['enableLinkTracking']);
(function() {{
    var u=`//{}/`;
    _paq.push(['setTrackerUrl', u+'matomo.php']);
    _paq.push(['setSiteId', '{}']);
    var d=document, g=d.createElement('script'), s=d.getElementsByTagName('script')[0];
    g.async=true; g.src=u+'matomo.js'; s.parentNode.insertBefore(g,s);
}})();", domain, env::var("ANALYTICS_ID").unwrap_or("1".to_string())),
                Err(_) => "".to_string(),
            };
            let a = NamedFile::open("./static/index.js");
            let mut a = a.expect("couldn't open index.js");
            let mut buf = String::new();
            let _ = a.read_to_string(&mut buf);
            HttpResponse::Ok().body(format!("{}{}", extra_text, buf))
        }
        #[get("/{game}/{player}")]
        async fn game_page(path: web::Path<(String, String)>) -> impl Responder {
            use std::io::Read;
            let a = NamedFile::open("./static/game.html");
            let mut a = a.expect("couldn't open game.html");
            let mut buf = String::new();
            let _ = a.read_to_string(&mut buf);
            let json = serde_json::to_string(&path.into_inner()).expect("couldn't serialize path");
            HttpResponse::Ok().body(format!("<script>const path_vars=JSON.parse(`{}`)</script>{}", json, buf))
        }
        /*
        Title: Leptos Counter Isomorphic Example
        Author:  Greg Johnston
        Date: Jan 2 2023
        Availability: https://github.com/leptos-rs/leptos/tree/683511f311e67ac979a4680ed1c01a0ee4272aa6/examples/counter_isomorphic
        Version: 0.2.5
        Note: The main portion of the code which this function is based on is line 14-29 in https://github.com/leptos-rs/leptos/blob/683511f311e67ac979a4680ed1c01a0ee4272aa6/examples/counter_isomorphic/src/main.rs
        */
        // This code allows the client to subscribe to a stream of events from the server
        #[get("/events/{game}/{player}")]
        async fn events(path: web::Path<(String,String)>) -> impl Responder {
            use loveletter::events::*;
            let (game, player) = path.into_inner();
            let game2 = game.clone();
            let mut conn = db::db().await.expect("couldn't connect to DB");
            let player = sqlx::query_as::<_, db::Players>("SELECT * FROM Players WHERE game=? AND id=?")
                    .bind(&game)
                    .bind(player)
                    .fetch_one(&mut conn)
                    .await;
            if player.is_err() {
                return HttpResponse::BadRequest().body("Invalid player");
            }
            let player = player.unwrap();
            let turn = player.turn;
            let text = player.text;
            let input = player.input;
            let mut first = true;
            let stream = futures::stream::once(async move {
                Event {
                game: game.clone(),
                player: None,
                event: EventData {
                    text,
                    input,
                },
            } })
                .chain(GAME_EVENTS.clone())
                .map(move |value:Event| {
                    let value = value.clone();
                    // Sends the event to the client if it is for the correct game and player
                    if value.game == game2 && value.player.unwrap_or(turn) == turn  {
                        if first {
                            first = false;
                        }
                        let json = serde_json::to_string(&value.event).unwrap();
                        Ok(web::Bytes::from(format!(
                            "{json}\n"
                        ))) as Result<web::Bytes>
                    } else {
                        Ok(web::Bytes::from("")) as Result<web::Bytes>
                    }
                });
            HttpResponse::Ok()
                .insert_header(("Content-Type", "text/event-stream"))
                .streaming(stream)
        }
        // This code allows the client to send input
        #[derive(Deserialize)]
        struct Input {
            input: String,
        }
        #[post("/input/{game}/{player}")]
        async fn send_input(body: web::Json<Input>, path: web::Path<(String,String)>) -> impl Responder {
            let (game_id, player_id) = path.into_inner();
            match game::recieve_input(game_id, player_id, &body.input).await {
                    Ok(_) => {
                        HttpResponse::Ok().body("Input received")
                    },
                    Err(response) => {
                        HttpResponse::BadRequest().body(response)
                    }
                }
        }
        // This code allows the client to create a game
        #[derive(Deserialize)]
        struct NewGame {
            players: i32,
        }
        #[post("/newgame")]
        async fn new_game(body: web::Json<NewGame>) -> impl Responder {
            let game_id = loveletter::random_id();
            let deck = serde_json::to_string(&loveletter::game::create_deck()).unwrap();
            let mut conn = db::db().await.expect("couldn't connect to DB");
            let _ = sqlx::query("INSERT OR IGNORE INTO Games (id, turn, deck, players) VALUES (?, 0, ?, ?)")
                .bind(&game_id)
                .bind(&deck)
                .bind(&body.players)
                .execute(&mut conn)
                .await
                .expect("couldn't insert game");
            #[cfg(not(feature = "client"))]
            println!("Server: Game {} created", game_id);
            HttpResponse::Ok().body(format!("{}", game_id))
        }
        // This code allows the client to join a game
        #[derive(Deserialize)]
        struct JoinGame {
            name: String,
        }
        #[post("/join/{game}")]
        async fn join_game(path: web::Path<(String,)>, body: web::Json<JoinGame>) -> impl Responder {
            let (game_id,) = path.into_inner();
            let player_id:String = loveletter::random_id();
            let mut conn = db::db().await.expect("couldn't connect to DB");
            let exists = sqlx::query_as::<_, db::Games>("SELECT * FROM games WHERE id=?")
                .bind(&game_id)
                .fetch_one(&mut conn)
                .await;
            if exists.is_err() {
                return HttpResponse::NotFound().body("Game not found");
            }
            let count = sqlx::query_as::<_, db::Players>("SELECT * FROM players WHERE game=?")
                .bind(&game_id)
                .fetch_all(&mut conn)
                .await
                .expect("couldn't get players")
                .len();
            let exists = exists.unwrap();
            // Makes sure there are not too many players
            if exists.players as usize <= count {
                return HttpResponse::BadRequest().body("Game is full");
            }
            let _ = sqlx::query("INSERT INTO Players (id, turn, game, hand, name, text) VALUES (?, (SELECT IFNULL(MAX(turn), 0) FROM players WHERE game=?)+1, ?, ?, ?, \"\")")
                .bind(&player_id)
                .bind(&game_id)
                .bind(&game_id)
                .bind(loveletter::game::create_hand(&game_id).await)
                .bind(&body.name)
                .execute(&mut conn)
                .await
                .expect("couldn't insert player");
            let _ = GAME_EVENTS.send(&Event {
                game: game_id.clone(),
                player: None,
                event: EventData {
                    text: format!("{} joined the game\n", body.name),
                    input: false,
                },
            }).await;
            let _ = GAME_EVENTS.send(&Event {
                game: game_id.clone(),
                player: None,
                event: EventData {
                    text: format!("To join the game use game ID {}\n", game_id),
                    input: false,
                },
            }).await;
            if exists.players as usize == count + 1 {
                loveletter::game::start_game(game_id).await;
            }
            HttpResponse::Ok().body(format!("{}", player_id))
        }
        // Used to start the web server
        #[actix_web::main]
        pub async fn main() -> std::io::Result<()> {
            #[cfg(feature = "client")]
            {
                use std::thread;
                thread::spawn(|| {
                    client();
                });
            }
            // Creates the database and runs the migrations
            use sqlx::{Sqlite, migrate::MigrateDatabase};
            std::fs::create_dir("./db").unwrap_or(());
            let _ = Sqlite::create_database("./db/db.db").await;
            let mut conn = loveletter::db::db().await.expect("couldn't connect to DB");
            // Used to start the event recorder
            tokio::task::spawn(async {
                loveletter::events::store_events().await;
            });
            sqlx::migrate!()
                .run(&mut conn)
                .await
                .expect("could not run SQLx migrations");
            println!("Server: Starting server on port 8080 you can open webui on http://localhost:8080...");
            HttpServer::new(|| {
                App::new().service(events).service(join_game).service(new_game).service(send_input).service(game_page).service(javascript).service(static_files)
            })
            .bind(("0.0.0.0", 8080))?
            .run()
            .await
        }
    } else {
        fn main() {
            client();
        }
    }
}
// Starts the client code
#[cfg(feature = "client")]
#[allow(dead_code)]
#[tokio::main]
async fn client() {
    use loveletter::client;
    client::main().await;
}
