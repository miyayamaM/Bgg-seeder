-- Add migration script here
CREATE TABLE IF NOT EXISTS boardgames(
    id INT,
    `name` VARCHAR(255),
    published_year INT,
    boardgame_geek_rank INT,
    average_rating DECIMAL(5,2),
    bayes_average_rating DECIMAL(5,2),
    users_rated INT,
    boardgame_geek_url TEXT,
    thumbnail_url TEXT
);
