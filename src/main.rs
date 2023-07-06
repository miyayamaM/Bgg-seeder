use anyhow::Result;
use sqlx::{Connection, MySqlConnection};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use serde::Deserialize;

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

#[tokio::main]
async fn main() -> Result<()> {
    // GitHubからcsvを取得
    let url =
        "https://raw.githubusercontent.com/beefsack/bgg-ranking-historicals/master/2023-06-29.csv";
    let mut body = reqwest::get(url).await?.text().await?;

    let csf_file = File::create("2023-06-29.csv")?;
    let mut writer = BufWriter::new(csf_file);
    let header = "id,name,published_year,boardgame_geek_rank,average_rating,bayes_average_rating,users_rated,boardgame_geek_url,thumbnail_url\n";
    writer.write(header.as_bytes())?;
    if let Some(index) = body.find('\n') {
        body.replace_range(..=index, ""); // 最初の改行までの文字列を削除
    }
    writer.write(body.as_bytes())?;

    // DBと接続
    let mut connection =
        MySqlConnection::connect("mysql://root:@localhost:3306/bgg_seeder").await?;

    // csvを解析してDBへいれる
    let file = File::open("2023-06-29.csv")?;
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
