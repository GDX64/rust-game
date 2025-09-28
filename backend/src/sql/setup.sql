create table if not exists kills (
    id integer primary key,
    frame integer,
    game_id integer,
    ship_id integer,
    player_id integer,
    killed_by integer
);

create table if not exists players(
  id integer primary key,
  game_id integer,
  name text,
  player_id integer,
  unique (player_id, game_id),
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

create table if not EXISTS servers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    seed INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

create table if not exists server_ticks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER not null,
    tick INTEGER not null,
    micros_elapsed INTEGER not null,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
);