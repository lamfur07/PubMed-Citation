use std::io;
use anyhow::Result;
use reqwest::Url;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    let mut term = String::new();
    println!("Please enter a search for PubMed articles: ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut term)
        .expect("Failed to read line");
    term = term.replace(" ", "+").trim().to_string();

    pubmed_search(term).await?;

    Ok(())
}

async fn pubmed_search(s: String) -> Result<()> {
    let base_url = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/";

    let database = "pubmed";

    let query_url = Url::parse_with_params(
        &format!("{}esearch.fcgi", base_url),
        &[
            ("db", database),
            ("term", &s),
            ("retmode", "json"),
        ],
    )?;

    let response: HashMap<String, Value> = reqwest::get(query_url)
        .await?
        .json::<HashMap<String, Value>>()
        .await?;

    let id_list = response.get("esearchresult")
        .and_then(|res| res.get("idlist"))
        .and_then(|idlist| idlist.as_array())
        .ok_or_else(|| anyhow::anyhow!("No articles found! Please try another search term."))?;

    if id_list.is_empty() {
        println!("No articles found! Please try another search term.");
    } else {
        let total = id_list.len().min(5);

        for (i, article) in id_list[0..total].iter().enumerate() {
            if i == 0 {
                println!("------------------------");
                println!("Beginning article search");
                println!("------------------------");
            }

            println!("Retrieving articles {} of {}", i + 1, total);

            let search_url = Url::parse_with_params(
                &format!("{}esummary.fcgi", base_url),
                &[
                    ("db", database),
                    ("id", article.as_str().unwrap()),
                    ("retmode", "json"),
                ],
            )?;

            let search_response: HashMap<String, Value> = reqwest::get(search_url)
                .await?
                .json::<HashMap<String, Value>>()
                .await?;

            let article_str = article.as_str().unwrap();
            let result = search_response.get("result")
                .and_then(|res| res.get(article_str))
                .ok_or_else(|| anyhow::anyhow!("No information found. Skipping.."))?;

            let authors = result.get("authors")
                .and_then(|authors| authors.as_array())
                .ok_or_else(|| anyhow::anyhow!("No authors found. Skipping.."))?;

            let mut author_list = vec![];

            for author in authors {
                let name = author.get("name")
                    .and_then(|name| name.as_str())
                    .ok_or_else(|| anyhow::anyhow!("No name found for author. Skipping.."))?;
                author_list.push(name);
            }

            let names = author_list.join(", ");

            let title = result.get("title")
                .and_then(|title| title.as_str())
                .unwrap_or("");

            let corrected_title = title.replace("&lt;i&gt;", "<i>").replace("&lt;/i&gt;", "</i>");

            let journal = result.get("source")
                .and_then(|source| source.as_str())
                .unwrap_or("");

            let pub_date = result.get("pubdate")
                .and_then(|pubdate| pubdate.as_str())
                .unwrap_or("");

            let volume = result.get("volume")
                .and_then(|volume| volume.as_str())
                .unwrap_or("");

            let issue = result.get("issue")
                .and_then(|issue| issue.as_str())
                .unwrap_or("");

            let pages = result.get("pages")
                .and_then(|pages| pages.as_str())
                .unwrap_or("");

            let doi = result.get("elocationid")
                .and_then(|elocationid| elocationid.as_str())
                .unwrap_or("");

            println!("PubMed ID: {}", article_str);
            println!("{} {}. {} {}; {}({}): {}. {}", names, corrected_title, journal, pub_date, volume, issue, pages, doi);

            println!("------------------------");
        }
    }

    Ok(())
}
