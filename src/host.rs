use std::collections::BTreeMap;

use rocket::response::status::BadRequest;
use rocket::tokio::runtime::Runtime;
use rocket::{get, routes};

use crate::app_config::APP_DISPLAY_NAME;
use crate::hsk_file::HskResult;
use crate::utils::Timer;
use crate::{CONFIG, SEARCHER};

pub fn command_host() -> HskResult<()> {
    // Create a new tokio runtime
    let rt = Runtime::new()?;
    // force it to load when starting
    {
        let mut timer = Timer::new();
        let searcher = SEARCHER.as_ref();
        timer.print(
            format!(
                "Searcher loaded {} transcripts",
                searcher.transcript_words.len(),
            )
            .as_str(),
        );
    }

    // Launch rocket in the runtime
    _ = rt.block_on(async {
        let rocket = rocket::build().mount("/", routes![index, search, ids]);
        rocket
            .launch()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        Ok::<(), Box<dyn std::error::Error>>(())
    })?;
    Ok(())
}

// Example route handlers
#[get("/")]
async fn index() -> &'static str {
    APP_DISPLAY_NAME
}

#[get("/ids")]
async fn ids() -> String {
    serde_json::to_string(
        &SEARCHER
            .transcript_paths
            .iter()
            .enumerate()
            .map(|(id, path)| (id, path))
            .collect::<BTreeMap<_, _>>(),
    )
    .expect("This can serialize")
}

#[get("/search?<query>&<context>&<page>&<remove_stop_words>")]
async fn search(
    query: String,
    context: Option<usize>,
    page: Option<usize>,
    remove_stop_words: bool,
) -> Result<String, BadRequest<String>> {
    let page = page.unwrap_or(0);
    let page_results = SEARCHER.search(
        query,
        context.unwrap_or(CONFIG.context_size()),
        page,
        remove_stop_words,
    );
    serde_json::to_string(&page_results).map_err(|err| BadRequest(err.to_string()))
}
