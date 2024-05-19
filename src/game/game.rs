use serde::Serialize;

use super::deck::{Card, Deck};

pub enum RoundAction {
    PlayCard(Player, Card),
    Truco(Player),
}

#[derive(Clone, Debug, Serialize)]
pub struct Player {
    pub name: String,
    pub id: u64,
}

impl Player {
    pub fn new(name: String, id: u64) -> Self {
        Self { name, id }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct GamePlayer {
    player: Player,
    hand: Vec<Card>,
}

impl GamePlayer {
    pub fn has_card(&self, card: &Card) -> bool {
        self.hand.contains(card)
    }

    pub fn remove_card(&mut self, card: &Card) {
        self.hand.retain(|c| c != card);
    }

    pub fn first_card(&self) -> Card {
        self.hand[0]
    }
}

#[derive(Debug)]
struct Round {
    players: [GamePlayer; 4],
    who_plays: usize,
    round_value: usize,
    points_team_1: usize,
    points_team_2: usize,
    turn: usize,
    cards_on_table: Vec<(Card, u64)>,
}

impl Round {
    fn new(players: [Player; 4]) -> Self {
        let mut deck = Deck::new();
        deck.shuffle();
        let players = players
            .into_iter()
            .map(|player| {
                GamePlayer {
                    player: player.clone(),
                    hand: deck.draw(3),
                }
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Self {
            players,
            who_plays: 0,
            turn: 0,
            round_value: 1,
            points_team_1: 0,
            points_team_2: 0,
            cards_on_table: Vec::new(),
        }
    }

    pub fn play(&mut self, action: RoundAction) -> anyhow::Result<()> {
        match action {
            RoundAction::PlayCard(player, card) => {
                if !self.player_has_card(&player, &card) {
                    return Err(anyhow::anyhow!("Player does not have this card"));
                }
                if !self.is_players_turn(&player) {
                    return Err(anyhow::anyhow!("It's not this player's turn"));
                }
                self.remove_card_from_player(&player, &card);
                self.cards_on_table.push((card, player.id));
                let ended_turn = self.cards_on_table.len() == 4;
                if !ended_turn {
                    self.who_plays += 1;
                    self.who_plays %= 4;
                    return Ok(());
                }

                let winner = self
                    .cards_on_table
                    .iter()
                    .max_by_key(|(card, _)| card.card_power())
                    .unwrap()
                    .1;
                let winner_player = self.players.iter().find(|p| p.player.id == winner).unwrap();
                self.who_plays = self
                    .players
                    .iter()
                    .position(|p| p.player.id == winner_player.player.id)
                    .unwrap();
                self.cards_on_table.clear();
                self.turn += 1;
                let winner_team = self.who_plays % 2;
                if winner_team == 0 {
                    self.points_team_1 += 1;
                } else {
                    self.points_team_2 += 1;
                }

                let ended_round = self.turn == 3;
                if ended_round {}
            }
            RoundAction::Truco(player) => {
                if !self.is_players_turn(&player) {
                    return Err(anyhow::anyhow!("It's not this player's turn"));
                }
                self.round_value += 1;
            }
        }
        Ok(())
    }

    fn player_now(&self) -> &GamePlayer {
        &self.players[self.who_plays]
    }

    fn is_players_turn(&self, player: &Player) -> bool {
        self.players[self.who_plays].player.id == player.id
    }

    fn remove_card_from_player(&mut self, player: &Player, card: &Card) {
        let player = self
            .players
            .iter_mut()
            .find(|p| p.player.id == player.id)
            .unwrap();

        player.remove_card(card)
    }

    fn player_has_card(&self, player: &Player, card: &Card) -> bool {
        self.players
            .iter()
            .find(|p| p.player.id == player.id)
            .unwrap()
            .has_card(card)
    }
}

pub struct Game {
    players: [Player; 4],
    round: Round,
    points_team1: usize,
    points_team2: usize,
}

impl Game {
    pub fn new(players: [Player; 4]) -> Self {
        Self {
            round: Round::new(players.clone()),
            players,
            points_team1: 0,
            points_team2: 0,
        }
    }

    pub fn play(&mut self, action: RoundAction) -> anyhow::Result<()> {
        self.round.play(action)
    }
}

mod test {
    use super::{Player, Round, RoundAction};

    fn test_players() -> [Player; 4] {
        let p1: Player = Player {
            name: "Player 1".to_string(),
            id: 1,
        };
        let p2: Player = Player {
            name: "Player 2".to_string(),
            id: 2,
        };
        let p3: Player = Player {
            name: "Player 3".to_string(),
            id: 3,
        };
        let p4: Player = Player {
            name: "Player 4".to_string(),
            id: 4,
        };
        [p1, p2, p3, p4]
    }

    #[test]
    fn test_round() {
        let p1 = test_players()[0].clone();
        let p2 = test_players()[1].clone();
        let mut round = Round::new(test_players());

        let card1 = round.players[1].hand[0];
        let result = round.play(RoundAction::PlayCard(p2, card1));
        assert!(result.is_err());

        let card1 = round.players[0].hand[0];
        let result = round.play(RoundAction::PlayCard(p1, card1));
        assert!(result.is_ok());
    }

    #[test]
    fn every_body_plays() {
        let mut round = Round::new(test_players());

        for i in 0..12 {
            let player = round.player_now();
            let _result = round.play(RoundAction::PlayCard(
                player.player.clone(),
                player.first_card(),
            ));
        }

        assert_eq!(round.points_team_1 + round.points_team_2, 3);
    }
}
