use anyhow::Result;
use serde::Deserialize;
use sqlx::{Connection, MySqlConnection};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use xml::reader::{EventReader, XmlEvent};

const BOARDGAMEGEEK_XML_API_ENDPOINT: &str = "https://boardgamegeek.com/xmlapi/boardgame/";

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
    let client = reqwest::Client::new();

    for record in rdr.deserialize() {
        let boardgame: Boardgame = record?;

        //boardgamegeek APIからプレイ人数とプレイ時間を取得
        let response = client
            .get(format!(
                "{}{}",
                BOARDGAMEGEEK_XML_API_ENDPOINT,
                boardgame.id.to_string()
            ))
            .query(&[("stats", "1")])
            .send()
            .await?
            .text()
            .await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

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
                    thumbnail_url,
                    min_players,
                    max_players,
                    min_playing_time,
                    max_playing_time,
                    average_weight
                )
                VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?);
            "#,
            boardgame.id,
            boardgame.name,
            boardgame.published_year,
            boardgame.boardgame_geek_rank,
            boardgame.average_rating,
            boardgame.bayes_average_rating,
            boardgame.users_rated,
            boardgame.boardgame_geek_url,
            boardgame.thumbnail_url,
            get_value_from_xml(&response, "minplayers"),
            get_value_from_xml(&response, "maxplayers"),
            get_value_from_xml(&response, "minplaytime"),
            get_value_from_xml(&response, "maxplaytime"),
            get_value_from_xml(&response, "averageweight")
        )
        .execute(&mut connection)
        .await?;
    }
    Ok(())
}

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

fn get_value_from_xml(xml_text: &str, target_key: &str) -> Option<String> {
    let parser = EventReader::new(xml_text.as_bytes());
    let mut is_found = false;

    for element in parser {
        match element {
            Ok(XmlEvent::StartElement { name, .. }) => {
                if name.local_name == target_key {
                    is_found = true;
                }
            }
            Ok(XmlEvent::Characters(xml_value)) => {
                if is_found {
                    return Some(xml_value);
                }
            }
            _ => {}
        }
    }
    None
}
