use game_state::{CanvasGame, GameMessage};

pub async fn run(mut channel: tokio::sync::mpsc::Receiver<GameMessage>) {
    let mut game = CanvasGame::new();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                // I will do some game logic here later
            }
            Some(msg) = channel.recv() => {
                match game.on_message(msg).await  {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
            }
        }
    }
}
