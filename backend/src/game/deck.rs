use std::fmt::Debug;

use serde::Serialize;

#[derive(PartialEq, Debug, Clone, Copy, Serialize)]
enum CardSuit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
enum CardValue {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

#[derive(PartialEq, Clone, Copy, Serialize)]
pub struct Card {
    value: CardValue,
    suit: CardSuit,
}

impl Into<String> for Card {
    fn into(self) -> String {
        let card = match self.value {
            CardValue::Ace => "A",
            CardValue::Two => "2",
            CardValue::Three => "3",
            CardValue::Four => "4",
            CardValue::Five => "5",
            CardValue::Six => "6",
            CardValue::Seven => "7",
            CardValue::Eight => "8",
            CardValue::Nine => "9",
            CardValue::Ten => "T",
            CardValue::Jack => "J",
            CardValue::Queen => "Q",
            CardValue::King => "K",
        };
        let suit = match self.suit {
            CardSuit::Hearts => "H",
            CardSuit::Diamonds => "D",
            CardSuit::Clubs => "C",
            CardSuit::Spades => "S",
        };
        format!("{}{}", card, suit)
    }
}

impl Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<String>::into(*self))
    }
}

impl From<String> for Card {
    fn from(value: String) -> Self {
        let mut chars = value.chars();
        let value = match chars.next() {
            Some('A') => CardValue::Ace,
            Some('2') => CardValue::Two,
            Some('3') => CardValue::Three,
            Some('4') => CardValue::Four,
            Some('5') => CardValue::Five,
            Some('6') => CardValue::Six,
            Some('7') => CardValue::Seven,
            Some('8') => CardValue::Eight,
            Some('9') => CardValue::Nine,
            Some('T') => CardValue::Ten,
            Some('J') => CardValue::Jack,
            Some('Q') => CardValue::Queen,
            Some('K') => CardValue::King,
            _ => panic!("Invalid card value"),
        };
        let suit = match chars.next() {
            Some('H') => CardSuit::Hearts,
            Some('D') => CardSuit::Diamonds,
            Some('C') => CardSuit::Clubs,
            Some('S') => CardSuit::Spades,
            _ => panic!("Invalid card suit"),
        };
        Self { value, suit }
    }
}

const ZAP_STRENTH: u8 = 100;
const SPADILHA_STRENGTH: u8 = 80;
const COPAO_STRENGTH: u8 = 90;
const OURINHOS_STRENGTH: u8 = 70;

impl Card {
    pub fn card_power(&self) -> u8 {
        match self.value {
            CardValue::Ace => {
                match self.suit {
                    CardSuit::Hearts => 11,
                    CardSuit::Diamonds => 11,
                    CardSuit::Clubs => 11,
                    CardSuit::Spades => SPADILHA_STRENGTH,
                }
            }
            CardValue::Two => 12,
            CardValue::Three => 13,
            CardValue::Four => {
                if self.suit == CardSuit::Clubs {
                    ZAP_STRENTH
                } else {
                    1
                }
            }
            CardValue::Five => 2,
            CardValue::Six => 3,
            CardValue::Seven => {
                match self.suit {
                    CardSuit::Hearts => COPAO_STRENGTH,
                    CardSuit::Diamonds => OURINHOS_STRENGTH,
                    CardSuit::Clubs => 4,
                    CardSuit::Spades => 4,
                }
            }
            CardValue::Eight => 5,
            CardValue::Nine => 6,
            CardValue::Ten => 7,
            CardValue::Jack => 8,
            CardValue::Queen => 9,
            CardValue::King => 10,
        }
    }
}

const ALL_CARDS: [&str; 44] = [
    "AH", "2H", "3H", "4H", "5H", "6H", "7H", "TH", "JH", "QH", "KH", "AD", "2D", "3D", "4D", "5D",
    "6D", "7D", "TD", "JD", "QD", "KD", "AC", "2C", "3C", "4C", "5C", "6C", "7C", "TC", "JC", "QC",
    "KC", "AS", "2S", "3S", "4S", "5S", "6S", "7S", "TS", "JS", "QS", "KS",
];

fn create_deck() -> Vec<Card> {
    ALL_CARDS
        .iter()
        .map(|c| Card::from(c.to_string()))
        .collect()
}

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let cards = create_deck();
        Self { cards }
    }

    pub fn shuffle(&mut self) {
        let mut rng = fastrand::Rng::new();
        rng.shuffle(&mut self.cards);
    }

    pub fn deal(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn draw(&mut self, n: usize) -> Vec<Card> {
        let mut hand = Vec::new();
        for _ in 0..n {
            if let Some(card) = self.deal() {
                hand.push(card);
            }
        }
        hand
    }
}
