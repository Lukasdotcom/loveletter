use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature = "server")] {
        /*
        Title: Love is in the air - a Love Letter Review
        Author:  Jay Fung
        Date: Jun 9 2016
        Availability: https://boardgamegeek.com/thread/1588636/love-air-love-letter-review
        Note: This was for the descriptions and the amount of cards. Not any code.
        */
        // The number of each card
        const CARDS: [usize; 9] = [0, 5, 2, 2, 2, 2, 1, 1, 1];
        const CARD_NAME: [&str; 9] = [
            "Empty Card",
            "Soldier",
            "Clown",
            "Knight",
            "Priestess",
            "Wizard",
            "General",
            "Minister",
            "Princess",
        ];
        // The description of each card
        const CARD_DESCRIPTIONS: [&str; 9] = [
            "You should never have seen this card. If you did something went really badly wrong.",
            "Guess another players card. If you are right they are eliminated. You may not guess Soldier.",
            "Look at another players card.",
            "Compare your card with another players card. The player with the lower card is eleminated.",
            "Ignore all attacks from other players.",
            "Target another player to discard a card",
            "Swap your hand with another players hand.",
            "If your hand has a total of 12 or more you are eliminated.",
            "If you discard this card you are eliminated.",
        ];
        use std::cmp::Ordering;

        use rand::seq::SliceRandom;

        use crate::events::GAME_EVENTS;
        // Used to create a hand for a player
        pub async fn create_hand(game_id: &str) -> String {
            let mut db = crate::db::db().await.expect("couldn't connect to DB");
            let deck = sqlx::query_as::<_, crate::db::Games>("SELECT * FROM games WHERE id=?")
                .bind(&game_id)
                .fetch_one(&mut db)
                .await
                .expect("couldn't get game")
                .deck;
            let mut deck = serde_json::from_str::<Vec<usize>>(&deck).unwrap();
            let hand = vec![deck.pop().unwrap(), 0];
            let deck = serde_json::to_string(&deck).unwrap();
            sqlx::query("UPDATE games SET deck=? WHERE id=?")
                .bind(&deck)
                .bind(&game_id)
                .execute(&mut db)
                .await
                .expect("couldn't update deck");
            serde_json::to_string(&hand).unwrap()
        }
        // Used to create a deck and shuffle it
        pub fn create_deck() -> Vec<usize> {
            let mut deck: Vec<usize> = Vec::new();
            for (i, &card) in CARDS.iter().enumerate() {
                for _ in 0..card {
                    deck.push(i);
                }
            }
            /*
            Title: How do I create a Vec from a range and shuffle it?
            Author:  Vladimir Matveev
            Date: Sep 25 2014
            Availability: https://stackoverflow.com/questions/26033976/how-do-i-create-a-vec-from-a-range-and-shuffle-it
            */
            deck.shuffle(&mut rand::thread_rng());
            deck
        }
        // Used to start a game
        pub async fn start_game(id: String) {
            let _ = GAME_EVENTS
                .send(&crate::events::Event {
                    game: id.clone(),
                    player: None,
                    event: crate::events::EventData {
                        text: format!("Game is starting\n"),
                        input: false,
                    },
                })
                .await;
            run_turn(id).await;
        }
        // Used to recieve an input
        pub async fn recieve_input(game_id: String, player_id: String, input: &str) -> Result<(), String> {
            // Makes sure that the player can give input
            let mut conn = crate::db::db().await.expect("couldn't connect to DB");
            let player =
                sqlx::query_as::<_, crate::db::Players>("SELECT * FROM players WHERE id=? AND game=?")
                    .bind(&player_id)
                    .bind(&game_id)
                    .fetch_one(&mut conn)
                    .await;
            if player.is_err() {
                return Err("Player Does not exist".to_string());
            }
            let player = player.unwrap();
            let game = sqlx::query_as::<_, crate::db::Games>("SELECT * FROM Games WHERE id=?")
                .bind(&game_id)
                .fetch_one(&mut conn)
                .await;
            if game.is_err() {
                return Err("Game Does not exist".to_string());
            }
            let game = game.unwrap();
            if game.turn != player.turn {
                return Err("It is not your turn".to_string());
            }
            if !player.input {
                return Err("You can't input right now".to_string());
            }
            let input = input.parse::<usize>();
            if input.is_err() {
                return Err("Invalid input".to_string());
            }
            let input = input.unwrap();
            let hand = serde_json::from_str::<Vec<usize>>(&player.hand).expect("Invalid hand");
            // Checks if the player is picking a card
            if game.played_card == -1 {
                let pick = input-1;
                // Makes sure the pick is valid
                if pick > 1 {
                    return Err("Invalid card picked".to_string());
                }
                let mut finished = true;
                match hand[pick] {
                    // An empty card was played(Should never happen but just in case)
                    0 => {
                        let _ = GAME_EVENTS
                            .send(&crate::events::Event {
                                game: game_id.clone(),
                                player: None,
                                event: crate::events::EventData {
                                    text: format!(
                                        "Player {} called {} played an empty card. How? I have no clue.\n",
                                        player.turn, player.name
                                    ),
                                    input: false,
                                },
                            })
                            .await;
                    }
                    // The player picked priestess
                    4 => {
                        let _ = GAME_EVENTS
                            .send(&crate::events::Event {
                                game: game_id.clone(),
                                player: None,
                                event: crate::events::EventData {
                                    text: format!(
                                        "Player {} called {} played the priestess.\n",
                                        player.turn, player.name
                                    ),
                                    input: false,
                                },
                            })
                            .await;
                        let _ = sqlx::query("UPDATE players SET immune=1 WHERE game=? AND turn=?")
                            .bind(&game_id)
                            .bind(&player.turn)
                            .execute(&mut conn)
                            .await
                            .expect("couldn't update player immunity");
                    }
                    // The player picked Minister
                    7 => {
                        let _ = GAME_EVENTS
                            .send(&crate::events::Event {
                                game: game_id.clone(),
                                player: None,
                                event: crate::events::EventData {
                                    text: format!(
                                        "Player {} called {} played the minister.\n",
                                        player.turn, player.name
                                    ),
                                    input: false,
                                },
                            })
                            .await;
                    }
                    // The player picked Princess
                    8 => {
                        let _ = GAME_EVENTS
                            .send(&crate::events::Event {
                                game: game_id.clone(),
                                player: None,
                                event: crate::events::EventData {
                                    text: format!(
                                        "Player {} called {} played the princess and is out.\n",
                                        player.turn, player.name
                                    ),
                                    input: false,
                                },
                            })
                            .await;
                        let _ = sqlx::query("UPDATE players SET alive=0 WHERE game=? AND turn=?")
                            .bind(&game_id)
                            .bind(&player.turn)
                            .execute(&mut conn)
                            .await
                            .expect("couldn't elminate player");
                    }
                    // For all cards the player could have picked that require a target
                    _ => {
                        let players = sqlx::query_as::<_, crate::db::Players>(
                            "SELECT * FROM players WHERE game=? AND alive=1 AND turn!=?",
                        )
                        .bind(&game_id)
                        .bind(&player.turn)
                        .fetch_all(&mut conn)
                        .await
                        .expect("couldn't get players");
                        // Collects all the players in a string to send to the player
                        let player_text = "Players:".to_string()
                            + &players
                                .iter()
                                .map(|player| format!("\n{} {}", player.turn, player.name))
                                .collect::<String>();
                        let _ = sqlx::query("UPDATE Games SET played_card=? WHERE id=?")
                            .bind(pick as i32)
                            .bind(&game_id)
                            .execute(&mut conn)
                            .await
                            .expect("couldn't update game");
                        let _ = GAME_EVENTS
                            .send(&crate::events::Event {
                                game: game_id.clone(),
                                player: Some(player.turn),
                                event: crate::events::EventData {
                                    text: format!("What player do you want to target?\n{}\n", player_text),
                                    input: true,
                                },
                            })
                            .await;
                        finished = false;
                    }
                }
                // Checks if a card was played
                if finished {
                    let mut hand2 = hand.clone();
                    hand2[pick] = 0;
                    let _ = sqlx::query("UPDATE players SET hand=? WHERE game=? AND turn=?")
                        .bind(serde_json::to_string(&hand2).expect("Invalid hand"))
                        .bind(&game_id)
                        .bind(&player.turn)
                        .execute(&mut conn)
                        .await
                        .expect("couldn't update player");
                    run_turn(game_id).await;
                }
            // Checks if a player is picking a target
            } else if game.player_pick == -1 {
                let card_picked = sqlx::query_as::<_, crate::db::Games>("SELECT * FROM Games WHERE id=?")
                    .bind(&game_id)
                    .fetch_one(&mut conn)
                    .await
                    .expect("couldn't get game");
                let player_picked = input;
                let opponent = sqlx::query_as::<_, crate::db::Players>(
                    "SELECT * FROM players WHERE game=? AND turn=? AND alive=1",
                )
                .bind(&game_id)
                .bind(player_picked as i32)
                .fetch_one(&mut conn)
                .await;
                if opponent.is_err() {
                    return Err("Invalid player picked".to_string());
                }
                // Gets the opponent data
                let opponent = opponent.unwrap();
                let mut finished = true;
                let mut discarded_card = false;
                // Checks if the player picked is immune to attacks
                if opponent.immune {
                    let _ = GAME_EVENTS
                        .send(&crate::events::Event {
                            game: game_id.clone(),
                            player: Some(player.turn),
                            event: crate::events::EventData {
                                text: format!(
                                    "Player {} called {} tried to play card {} called {} on player {} called {}, but player {} is immune to attacks right now.\n",
                                    player.turn, player.name, card_picked.played_card, CARD_NAME[card_picked.played_card as usize], player_picked, opponent.name, player_picked
                                ),
                                input: false,
                            },
                        })
                        .await;
                } else {
                    match hand[card_picked.played_card as usize] {
                        // The player picked the clown
                        2 => {
                            // Gets strings of the cards the opponent has
                            let opponent_cards = serde_json::from_str::<Vec<i32>>(&opponent.hand)
                                .expect("Invalid hand")
                                .iter()
                                .filter(|card| **card != 0)
                                .map(|card| {
                                    format!(
                                        "\n{}-{}-{}",
                                        card, CARD_NAME[*card as usize], CARD_DESCRIPTIONS[*card as usize],
                                    )
                                })
                                .collect::<String>();
                            let _ = GAME_EVENTS
                                .send(&crate::events::Event {
                                    game: game_id.clone(),
                                    player: Some(player.turn),
                                    event: crate::events::EventData {
                                        text: format!(
                                            "Player {} called {} has card(s):{}\n",
                                            opponent.turn, opponent.name, opponent_cards
                                        ),
                                        input: false,
                                    },
                                })
                                .await;
                            let _ = GAME_EVENTS
                                .send(&crate::events::Event {
                                    game: game_id.clone(),
                                    player: None,
                                    event: crate::events::EventData {
                                        text: format!("Player {} called {} played the clown, and looked at player {} called {}\n", player.turn, player.name, opponent.turn, opponent.name),
                                        input: false,
                                    },
                                })
                                .await;
                        }
                        // The player picked the knight
                        3 => {
                            discarded_card = true;
                            let mut hand = hand.clone();
                            hand[card_picked.played_card as usize] = 0;
                            let _ = sqlx::query("UPDATE players SET hand=? WHERE game=? AND turn=?")
                                .bind(serde_json::to_string(&hand).expect("Invalid hand"))
                                .bind(&game_id)
                                .bind(&player.turn)
                                .execute(&mut conn)
                                .await
                                .expect("couldn't update player");
                            let mut opponent_card = 0;
                            for x in serde_json::from_str::<Vec<usize>>(&opponent.hand).expect("Invalid hand") {
                                if x > opponent_card {
                                    opponent_card = x;
                                }
                            }
                            let mut played_card = 0;
                            for x in hand {
                                if x > played_card {
                                    played_card = x;
                                }
                            }
                            // Checks who won and eliminates the loser
                            match played_card.cmp(&opponent_card) {
                                Ordering::Less => {
                                    let _ = GAME_EVENTS
                                        .send(&crate::events::Event {
                                            game: game_id.clone(),
                                            player: None,
                                            event: crate::events::EventData {
                                                text: format!("Player {} called {} played a knight and lost against player {} called {}.\nPlayer {} had a {}-{}\n", player.turn, player.name, opponent.turn, opponent.name, player.turn, played_card, CARD_NAME[played_card as usize]),
                                                input: false,
                                            },
                                        })
                                        .await;
                                    let _ =
                                        sqlx::query("UPDATE players SET alive=0 WHERE game=? AND turn=?")
                                            .bind(&game_id)
                                            .bind(&player.turn)
                                            .execute(&mut conn)
                                            .await
                                            .expect("couldn't update player");
                                }
                                Ordering::Equal => {
                                    let _ = GAME_EVENTS
                                        .send(&crate::events::Event {
                                            game: game_id.clone(),
                                            player: None,
                                            event: crate::events::EventData {
                                                text: format!("Player {} called {} played a knight and tied with player {} called {}.\n", player.turn, player.name, opponent.turn, opponent.name),
                                                input: false,
                                            },
                                        })
                                        .await;
                                }
                                Ordering::Greater => {
                                    let _ = GAME_EVENTS
                                        .send(&crate::events::Event {
                                            game: game_id.clone(),
                                            player: None,
                                            event: crate::events::EventData {
                                                text: format!("Player {} called {} played a knight and won against player {} called {}\nPlayer {} had a {}-{}\n", player.turn, player.name, opponent.turn, opponent.name, opponent.turn, opponent_card, CARD_NAME[opponent_card as usize]),
                                                input: false,
                                            },
                                        })
                                        .await;
                                    let _ =
                                        sqlx::query("UPDATE players SET alive=0 WHERE game=? AND turn=?")
                                            .bind(&game_id)
                                            .bind(&opponent.turn)
                                            .execute(&mut conn)
                                            .await
                                            .expect("couldn't update player");
                                }
                            }
                        }
                        // The player picked the wizard
                        5 => {
                            let mut deck =
                                serde_json::from_str::<Vec<usize>>(&game.deck).expect("Invalid deck");
                            let opponent_hand: Vec<usize> = vec![deck.pop().unwrap(), 0];
                            let _ = sqlx::query("UPDATE players SET hand=? WHERE game=? AND turn=?")
                                .bind(&serde_json::to_string(&opponent_hand).unwrap())
                                .bind(&game_id)
                                .bind(&opponent.turn)
                                .execute(&mut conn)
                                .await
                                .expect("couldn't update player");
                            let _ = sqlx::query("UPDATE Games SET deck=? WHERE id=?")
                                .bind(&serde_json::to_string(&deck).unwrap())
                                .bind(&game_id)
                                .execute(&mut conn)
                                .await
                                .expect("couldn't update game");
                            let _ = GAME_EVENTS
                                .send(&crate::events::Event {
                                    game: game_id.clone(),
                                    player: None,
                                    event: crate::events::EventData {
                                        text: format!("Player {} called {} played a wizard and caused player {} called {} to discard their hand\n", player.turn, player.name, opponent.turn, opponent.name),
                                        input: false,
                                    },
                                })
                                .await;
                        }
                        // The player picked the general
                        6 => {
                            discarded_card = true;
                            let mut hand = hand.clone();
                            hand[card_picked.played_card as usize] = 0;
                            let hand_text = serde_json::to_string(&hand).expect("Invalid hand");
                            let _ = sqlx::query("UPDATE players SET hand=? WHERE game=? AND turn=?")
                                .bind(&hand_text)
                                .bind(&game_id)
                                .bind(&player.turn)
                                .execute(&mut conn)
                                .await
                                .expect("couldn't update player");
                            let _ = sqlx::query("UPDATE Players SET hand=? WHERE game=? AND turn=?")
                                .bind(&hand_text)
                                .bind(&game_id)
                                .bind(opponent.turn)
                                .execute(&mut conn)
                                .await
                                .expect("coudn't update opponent");
                            let _ = sqlx::query("UPDATE Players SET hand=? WHERE game=? AND turn=?")
                                .bind(&opponent.hand)
                                .bind(&game_id)
                                .bind(player.turn)
                                .execute(&mut conn)
                                .await
                                .expect("coudn't update player");
                            let _ = GAME_EVENTS
                                .send(&crate::events::Event {
                                    game: game_id.clone(),
                                    player: None,
                                    event: crate::events::EventData {
                                        text: format!("Player {} called {} played a general and caused player {} called {} to switch hands with them\n", player.turn, player.name, opponent.turn, opponent.name),
                                        input: false,
                                    },
                                })
                                .await;
                            // Tells the 2 players the new cards they have
                            let played_cards = serde_json::from_str::<Vec<i32>>(&opponent.hand)
                                .expect("Invalid hand")
                                .iter()
                                .filter(|card| **card != 0)
                                .map(|card| {
                                    format!(
                                        "\n{}-{}-{}",
                                        card, CARD_NAME[*card as usize], CARD_DESCRIPTIONS[*card as usize],
                                    )
                                })
                                .collect::<String>();
                            let opponent_cards = hand
                                .iter()
                                .filter(|card| **card != 0)
                                .map(|card| {
                                    format!(
                                        "\n{}-{}-{}",
                                        card, CARD_NAME[*card as usize], CARD_DESCRIPTIONS[*card as usize],
                                    )
                                })
                                .collect::<String>();
                            let _ = GAME_EVENTS
                                .send(&crate::events::Event {
                                    game: game_id.clone(),
                                    player: Some(player.turn),
                                    event: crate::events::EventData {
                                        text: format!("Your cards were switched and now are {}\n", played_cards),
                                        input: false,
                                    },
                                })
                                .await;
                            let _ = GAME_EVENTS
                                .send(&crate::events::Event {
                                    game: game_id.clone(),
                                    player: Some(opponent.turn),
                                    event: crate::events::EventData {
                                        text: format!(
                                            "Your cards were switched and now are {}\n",
                                            opponent_cards
                                        ),
                                        input: false,
                                    },
                                })
                                .await;
                        }
                        // For the soldier lets the player pick a guess
                        _ => {
                            let _ = sqlx::query("UPDATE Games SET player_pick=? WHERE id=?")
                                .bind(player_picked as i32)
                                .bind(&game_id)
                                .execute(&mut conn)
                                .await
                                .expect("failed to save target");
                            let _ = GAME_EVENTS
                                .send(&crate::events::Event {
                                    game: game_id.clone(),
                                    player: Some(player.turn),
                                    event: crate::events::EventData {
                                        text: format!("Pick the card number you want to guess:\n"),
                                        input: true,
                                    },
                                })
                                .await;
                            finished = false;
                        }
                    }
                }
                if finished {
                    if !discarded_card {
                        let mut hand2 = hand;
                        hand2[card_picked.played_card as usize] = 0;
                        let _ = sqlx::query("UPDATE players SET hand=? WHERE game=? AND turn=?")
                            .bind(serde_json::to_string(&hand2).expect("Invalid hand"))
                            .bind(&game_id)
                            .bind(&player.turn)
                            .execute(&mut conn)
                            .await
                            .expect("couldn't update player");
                    }
                    
                    run_turn(game_id).await;
                }
            // Used to run the guess for the soldier
            } else {
                let guess = input;
                if guess <= 1 || guess > 8 {
                    return Err("Invalid guess".to_string());
                }
                // Removes the players card from their hand
                let mut player_hand =
                    serde_json::from_str::<Vec<usize>>(&player.hand).expect("Invalid hand");
                player_hand[game.played_card as usize] = 0;
                let _ = sqlx::query("UPDATE Players SET hand=? WHERE game=? AND turn=?")
                    .bind(&serde_json::to_string(&player_hand).unwrap())
                    .bind(&game_id)
                    .bind(&player.turn)
                    .execute(&mut conn)
                    .await
                    .expect("couldn't update player");
                let opponent = sqlx::query_as::<_, crate::db::Players>(
                    "SELECT * FROM Players WHERE game=? AND turn=?",
                )
                .bind(&game_id)
                .bind(&game.player_pick)
                .fetch_one(&mut conn)
                .await
                .expect("couldn't get player");
                let opponent_hand =
                    serde_json::from_str::<Vec<usize>>(&opponent.hand).expect("Invalid hand");
                /*
                Title: How do I check if a thing is in a vector
                Author: 8176135
                Date: Oct 14 2019
                Availability: https://stackoverflow.com/questions/58368801/how-do-i-check-if-a-thing-is-in-a-vector
                */
                // Checks if the guess was valid and correct
                if opponent_hand.iter().any(|&a| a == guess) {
                    // Eliminates the opponent
                    let _ = sqlx::query("UPDATE Players SET alive=0 WHERE game=? AND turn=?")
                        .bind(&game_id)
                        .bind(&opponent.turn)
                        .execute(&mut conn)
                        .await
                        .expect("coudn't update opponent");
                    let _ = GAME_EVENTS
                        .send(&crate::events::Event {
                            game: game_id.clone(),
                            player: None,
                            event: crate::events::EventData {
                                text: format!("Player {} called {} played a soldier, and guessed that player {} called {} had the card number {} called {} and was correct.\n", player.turn, player.name, opponent.turn, opponent.name, guess, CARD_NAME[guess]),
                                input: false,
                            },
                        })
                        .await;
                    let _ = GAME_EVENTS
                        .send(&crate::events::Event {
                            game: game_id.clone(),
                            player: None,
                            event: crate::events::EventData {
                                text: format!(
                                    "Player {} called {} is eliminated due to soldier.\n",
                                    opponent.turn, opponent.name
                                ),
                                input: false,
                            },
                        })
                        .await;
                } else {
                    let _ = GAME_EVENTS
                        .send(&crate::events::Event {
                            game: game_id.clone(),
                            player: None,
                            event: crate::events::EventData {
                                text: format!("Player {} called {} played a soldier, and guessed that player {} called {} had the card number {} called {} and was wrong.\n", player.turn, player.name, opponent.turn, opponent.name, guess, CARD_NAME[guess]),
                                input: false,
                            },
                        })
                        .await;
                }
                run_turn(game_id).await;
            }
            Ok(())
        }
        // Skips turns until you get to a player that is not eliminated
        pub async fn continue_turns(id: &str) {
            let mut conn = crate::db::db().await.expect("couldn't connect to DB");
            loop {
                let _ = sqlx::query(
                    "UPDATE GAMES SET turn=iif(turn>=(SELECT IFNULL(MAX(turn), 0) FROM players WHERE game=?), 1, Games.turn+1) WHERE id=?",
                )
                .bind(&id)
                .bind(&id)
                .execute(&mut conn)
                .await
                .expect("couldn't update turn");
                let game = sqlx::query_as::<_, crate::db::Games>("SELECT * FROM Games WHERE id=?")
                    .bind(&id)
                    .fetch_one(&mut conn)
                    .await
                    .expect("couldn't get player");
                let _ = sqlx::query("UPDATE players SET immune=0 WHERE game=? AND turn=?")
                    .bind(&id)
                    .bind(&game.turn)
                    .execute(&mut conn)
                    .await
                    .expect("couldn't update turn");
                let player = sqlx::query_as::<_, crate::db::Players>(
                    "SELECT * FROM players WHERE game=? AND turn=?",
                )
                .bind(&id)
                .bind(&game.turn)
                .fetch_one(&mut conn)
                .await
                .expect("couldn't get player");
                if player.alive {
                    break;
                }
            }
        }
        use async_recursion::async_recursion;
        // Used to run a turn
        #[async_recursion]
        pub async fn run_turn(id: String) {
            println!("Running turn for game {}...", id);
            let mut conn = crate::db::db().await.expect("couldn't connect to DB");
            // Checks how many players are left in the game
            let players =
                sqlx::query_as::<_, crate::db::Players>("SELECT * FROM players WHERE game=? AND alive=1")
                    .bind(&id)
                    .fetch_all(&mut conn)
                    .await
                    .expect("couldn't get players");
            let _ = sqlx::query("UPDATE Games SET player_pick=-1, played_card=-1  WHERE id=?")
                .bind(&id)
                .execute(&mut conn)
                .await
                .expect("couldn't update player pick");
            if players.len() == 0 {
                let _ = GAME_EVENTS
                    .send(&crate::events::Event {
                        game: id,
                        player: None,
                        event: crate::events::EventData {
                            text: format!("Game over! No players left! Something went wrong :(\n"),
                            input: false,
                        },
                    })
                    .await;
                return;
            } else if players.len() < 2 {
                let _ = GAME_EVENTS
                    .send(&crate::events::Event {
                        game: id,
                        player: None,
                        event: crate::events::EventData {
                            text: format!(
                                "Game over! Player {} called {} wins!\n",
                                players[0].turn, players[0].name
                            ),
                            input: false,
                        },
                    })
                    .await;
                return;
            }
            continue_turns(&id).await;
            let game = sqlx::query_as::<_, crate::db::Games>("SELECT * FROM Games WHERE id=?")
                .bind(&id)
                .fetch_one(&mut conn)
                .await
                .expect("couldn't get player");
            let player =
                sqlx::query_as::<_, crate::db::Players>("SELECT * FROM players WHERE game=? AND turn=?")
                    .bind(&id)
                    .bind(&game.turn)
                    .fetch_one(&mut conn)
                    .await
                    .expect("couldn't get player");
            let mut cards = serde_json::from_str::<Vec<usize>>(&player.hand).expect("couldn't parse cards");
            let mut deck = serde_json::from_str::<Vec<usize>>(&game.deck).expect("couldn't parse deck");
            // Checks if the deck is empty and the game should end
            if deck.len() == 0 {
                let mut winner: Option<crate::db::Players> = None;
                let mut biggest_card: usize = 0;
                for player in players {
                    let player_hand =
                        serde_json::from_str::<Vec<usize>>(&player.hand).expect("couldn't parse cards");
                    let new_biggest_card = player_hand.iter().max().unwrap_or(&0).to_owned();
                    if new_biggest_card > biggest_card {
                        winner = Some(player);
                        biggest_card = new_biggest_card;
                    }
                }
                if winner.is_some() {
                    let winner = winner.expect("couldn't get winner");
                    let _ = GAME_EVENTS
                        .send(&crate::events::Event {
                            game: id.clone(),
                            player: None,
                            event: crate::events::EventData {
                                text: format!("Player {} called {} won the game because they had the highest card {}-{}\n", winner.turn, winner.name, biggest_card, CARD_NAME[biggest_card]),
                                input: false,
                            },
                        })
                        .await;
                } else {
                    let _ = GAME_EVENTS
                        .send(&crate::events::Event {
                            game: id.clone(),
                            player: None,
                            event: crate::events::EventData {
                                text: format!("Somehow noone won the game :(\n"),
                                input: false,
                            },
                        })
                        .await;
                }
                return;
            }
            let _ = GAME_EVENTS
                .send(&crate::events::Event {
                    game: id.clone(),
                    player: None,
                    event: crate::events::EventData {
                        text: format!("It is player {} called {}'s turn\n", player.turn, player.name),
                        input: false,
                    },
                })
                .await;
            // Draws the next card for the player
            if cards[0] == 0 {
                cards[0] = deck.pop().unwrap_or(1);
            }
            if cards[1] == 0 {
                cards[1] = deck.pop().unwrap_or(1);
            }
            let card1 = cards[0];
            let card2 = cards[1];
            let _ = GAME_EVENTS
                .send(&crate::events::Event {
                    game: id.clone(),
                    player: Some(player.turn),
                    event: crate::events::EventData {
                        text: format!(
                            "Your hand is:\n1. {}-{}-{}\n2. {}-{}-{}\nDo you want to play card 1 or 2:\n",
                            card1,
                            CARD_NAME[card1],
                            CARD_DESCRIPTIONS[card1],
                            card2,
                            CARD_NAME[card2],
                            CARD_DESCRIPTIONS[card2]
                        ),
                        input: true,
                    },
                })
                .await;
            // Updates the deck and the hand
            let _ = sqlx::query("UPDATE Games SET deck=? WHERE id=?")
                .bind(&serde_json::to_string(&deck).expect("couldn't serialize deck"))
                .bind(&id)
                .execute(&mut conn)
                .await
                .expect("couldn't update game");
            let _ = sqlx::query("UPDATE players SET hand=? WHERE game=? AND turn=?")
                .bind(&serde_json::to_string(&cards).expect("couldn't serialize hand"))
                .bind(&id)
                .bind(&player.turn)
                .execute(&mut conn)
                .await
                .expect("couldn't update player");
            // Checks if the player is out due to the minister
            if (card1 == 7 || card2 == 7) && card2 >= 5 && card1 >= 5 {
                let _ = sqlx::query("UPDATE players SET alive=0 WHERE game=? AND turn=?")
                    .bind(&id)
                    .bind(&player.turn)
                    .execute(&mut conn)
                    .await
                    .expect("couldn't update player");
                let _ = GAME_EVENTS
                    .send(&crate::events::Event {
                        game: id.clone(),
                        player: None,
                        event: crate::events::EventData {
                            text: format!("Player {} called {} is out due to having a minister and a hand total of 12 or greater.\n", player.turn, player.name),
                            input: false,
                        },
                    })
                    .await;
                // Goes to the next player
                run_turn(id).await;
            }
        }
    }
}
