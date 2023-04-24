# Loveletter
This is just a simple server and a client multiplayer game of loveletter. Note that this was also my CSP performance task. To run this you need to have cargo installed and you can run.
```zsh
cargo run --release
```
That is for the client. For the server you can run the first command, or you can use docker and run the second command.
```zsh
cargo run --release --no-default-features -F server
docker run -d -p 8080:8080 lukasdotcom/loveletter
```
Then you can add that as the server to connect to in the client or just use the default server https://loveletter.lschaefer.xyz. Note that the server will automatically host a web ui and if you just want to try the program without an instalation visit https://loveletter.lschaefer.xyz.