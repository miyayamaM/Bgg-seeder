use anyhow::Result;
use serde::Deserialize;
use sqlx::{Connection, MySqlConnection};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

#[derive(Deserialize)]
struct Boardgame {
    id: u32,
    name: String,
    published_year: u32,
    boardgame_geek_rank: u32,
    average_rating: f32,
    bayes_average_rating: f32,
    users_rated: u32,
    boardgame_geek_url: String,
    thumbnail_url: String,
}

fn save_csv(file_name: &str, header: &str, response_body: String) -> Result<()> {
    let csv_file = File::create(file_name)?;
    let mut writer = BufWriter::new(csv_file);

    let mut copied_response_body = response_body.clone();
    if let Some(index) = copied_response_body.find('\n') {
        copied_response_body.replace_range(..=index, header);
    }

    writer.write(copied_response_body.as_bytes())?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // GitHubからcsvを取得
    let url =
        "https://raw.githubusercontent.com/beefsack/bgg-ranking-historicals/master/2023-06-29.csv";
    let response_body = reqwest::get(url).await?.text().await?;

    let header = "id,name,published_year,boardgame_geek_rank,average_rating,bayes_average_rating,users_rated,boardgame_geek_url,thumbnail_url\n";
    let saved_file_name = "bgg_ranking.csv";
    save_csv(saved_file_name, header, response_body)?;

    // DBと接続
    let mut connection =
        MySqlConnection::connect("mysql://root:@localhost:3306/bgg_seeder").await?;

    // csvを解析してDBへいれる
    let file = File::open(saved_file_name)?;
    let reader = BufReader::new(file);
    let mut rdr = csv::Reader::from_reader(reader);

    for record in rdr.deserialize() {
        let record: Boardgame = record?;

        sqlx::query!(
            r#"
                INSERT INTO boardgames(
                    id,
                    name,
                    published_year,
                    boardgame_geek_rank,
                    average_rating,
                    bayes_average_rating,
                    users_rated,
                    boardgame_geek_url,
                    thumbnail_url
                )
                VALUES(?,?,?,?,?,?,?,?,?);
            "#,
            record.id,
            record.name,
            record.published_year,
            record.boardgame_geek_rank,
            record.average_rating,
            record.bayes_average_rating,
            record.users_rated,
            record.boardgame_geek_url,
            record.thumbnail_url
        )
        .execute(&mut connection)
        .await?;
    }
    Ok(())
}
