use anyhow::Result;
use sqlx::{Connection, MySqlConnection};
use std::fs::File;
use std::io::{BufWriter, Write};

#[tokio::main]
async fn main() -> Result<()> {
    // GitHubからcsvを取得
    let url =
        "https://raw.githubusercontent.com/beefsack/bgg-ranking-historicals/master/2023-06-29.csv";
    let body = reqwest::get(url).await?.text().await?;

    let csf_file = File::create("2023-06-29.csv")?;
    let mut writer = BufWriter::new(csf_file);
    writer.write(body.as_bytes())?;

    // DBと接続
    let mut connection =
        MySqlConnection::connect("mysql://root:@localhost:3306/bgg_seeder").await?;

    sqlx::query(
        r#"
    INSERT INTO boardgames(
        id,
        name
    )
    VALUES(
        1,
        "SCOUT!"
    );
"#,
    )
    .execute(&mut connection)
    .await?;
    // csvを解析してDBへいれる

    Ok(())
}
