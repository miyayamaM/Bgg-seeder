-- Add migration script here
ALTER TABLE boardgames
ADD COLUMN min_players int,
ADD COLUMN max_players int,
ADD COLUMN min_playing_time int,
ADD COLUMN max_playing_time int,
ADD COLUMN average_weight DECIMAL(8,4);
