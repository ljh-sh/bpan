use crate::client::{Client, SearchOptions};
use crate::commands::GlobalOpts;
use crate::error::Result;

/// `bpan search <query>` — semantic search.
pub async fn run(client: &Client, opts: &GlobalOpts, query: String, dir: String, r#type: String) -> Result<()> {
    let category = match r#type.as_str() {
        "file" => Some(vec![1]),
        "dir" => Some(vec![0]),
        _ => None,
    };
    let search_opts = SearchOptions {
        dir,
        category,
        limit: 50,
    };
    let results = client.search(&query, search_opts).await?;

    if opts.human {
        for r in &results {
            println!("{}", r.filename);
        }
    } else {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "data": {
                    "query": query,
                    "results": results,
                    "total": results.len(),
                }
            })
        );
    }
    Ok(())
}