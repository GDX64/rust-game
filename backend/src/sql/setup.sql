
PRAGMA journal_mode
= WAL;
PRAGMA synchronous = OFF;
PRAGMA temp_store = MEMORY;

BEGIN;

    -- drop Table if EXISTS kills;
    -- drop Table if EXISTS players;
    -- drop Table if EXISTS servers;
    -- drop Table if EXISTS server_ticks;
    -- drop Table if EXISTS pings;
    -- drop table if EXISTS player_counts;
    -- drop table if EXISTS ships_destroyed;
    -- drop table if EXISTS ships_created;

    create table
    if not exists ships_destroyed
    (
    id integer primary key,
    frame integer,
    game_id integer,
    ship_id integer,
    owner integer,
    killer integer
);

    create table
    if not exists ships_created
    (
    id integer primary key,
    frame integer,
    game_id integer,
    ship_id integer,
    owner integer
);

create table
if not exists players
(
  id integer primary key,
  game_id integer,
  name text,
  player_id integer,
  tick integer,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

create table
if not EXISTS games
(
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    seed INTEGER NOT NULL,
    experiment_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

create table
if not exists game_ticks
(
    id INTEGER PRIMARY KEY,
    game_id INTEGER not null,
    tick INTEGER not null,
    micros_elapsed INTEGER not null,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
);

create table
if not exists experiment_ticks
(
    id INTEGER PRIMARY KEY,
    experiment_id INTEGER not null,
    tick INTEGER not null,
    micros_elapsed INTEGER not null,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
);

create table
if not exists pings
(
    id integer primary key,
    tick INTEGER not null,
    player_id integer not null,
    micros integer not null
);

create table
if not exists player_counts
(
    id integer primary key,
    game_id integer not null,
    tick integer not null,
    count integer not null,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

create table
if not exists experiments
(
    id integer primary key,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    number_of_bots integer not null,
    number_of_servers integer not null 
);

COMMIT;
VACUUM;