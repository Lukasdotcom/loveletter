#[cfg(feature = "client")]
pub async fn main() {
    use std::env;
    use std::io;
    /*
    Title: How can I access command line parameters in Rust?
    Author: barjak
    Date: Mar 25 2013
    Availability: https://stackoverflow.com/questions/15619320/how-can-i-access-command-line-parameters-in-rust
    */
    // Skips the game joinig process if a game ID is provided
    let args: Vec<_> = env::args().collect();
    if args.len() > 3 {
        event_stream(
            args[1].to_string(),
            args[2].to_string(),
            args[3].to_string(),
        )
        .await;
        return;
    }
    // Waits a second so if the server is enabled it shows the server message first
    #[cfg(feature = "server")]
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let mut server = String::new();
    println!("Enter the server address(ex:https://example.com): ");
    io::stdin()
        .read_line(&mut server)
        .expect("Failed to read line");
    let server = server.trim();
    let mut create = String::new();
    println!("Do you want to create a new game? (y/n): ");
    io::stdin()
        .read_line(&mut create)
        .expect("Failed to read line");
    let create = create.chars().next().unwrap() == 'y';
    // Used to create or get a game ID
    let game_id = if create {
        let mut players = String::new();
        println!("Enter the number of players you are going to play with: ");
        io::stdin()
            .read_line(&mut players)
            .expect("Failed to read line");
        let players = players.trim().parse::<i32>().unwrap();
        let client = reqwest::Client::new();
        client
            .post(format!("{}/newgame", server))
            .header("Content-Type", "application/json")
            .body(format!("{{\"players\":{}}}", players))
            .send()
            .await
            .expect("Couldn't connect to server")
            .text()
            .await
            .expect("Couldn't get game ID")
    } else {
        let mut game = String::new();
        println!("Enter the game ID: ");
        io::stdin()
            .read_line(&mut game)
            .expect("Failed to read line");
        game
    };
    let mut name = String::new();
    println!("Enter your name: ");
    io::stdin()
        .read_line(&mut name)
        .expect("Failed to read line");
    let name = name.trim();
    let body = serde_json::json!({ "name": name }).to_string();
    // Joins the game and subscribes to the event stream
    let client = reqwest::Client::new();
    let player_id = client
        .post(format!("{}/join/{}", server, game_id))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .expect("Couldn't connect to server")
        .text()
        .await
        .expect("Couldn't get player ID");
    event_stream(server.to_string(), game_id, player_id).await;
}

#[cfg(feature = "client")]
async fn event_stream(server: String, game_id: String, player_id: String) {
    use std::io;
    println!("If you want to run this program again while staying in the same game you can just run the command and add `{} {} {}` to the end.", server, game_id, player_id);
    let link = format!("{}/events/{}/{}", server, game_id, player_id);
    let mut event_stream = reqwest::get(link)
        .await
        .expect("Couldn't connect to server");
    while let Some(item) = event_stream.chunk().await.expect("chunk error") {
        let item = std::str::from_utf8(&item).unwrap().trim();
        let event = serde_json::from_str::<crate::events::EventData>(item).unwrap();
        print!("{}", event.text);
        // Checks if the event is an input event
        while event.input {
            println!("Enter your input:");
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let input = input.trim();
            let body = serde_json::json!({ "input": input }).to_string();
            let client = reqwest::Client::new();
            let res = client
                .post(format!("{}/input/{}/{}", server, game_id, player_id))
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .await
                .expect("Couldn't connect to server")
                .text()
                .await
                .expect("Couldn't send input");
            if res.eq("Input received") {
                break;
            } else {
                println!("{}", res);
            }
        }
    }
}
