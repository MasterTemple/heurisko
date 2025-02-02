use std::collections::BTreeMap;

use rocket::response::status::BadRequest;
use rocket::tokio::runtime::Runtime;
use rocket::{get, post, routes};
use serde::{Deserialize, Serialize};

use crate::app_config::APP_DISPLAY_NAME;
use crate::hsk_file::HskResult;
use crate::utils::Timer;
use crate::{CONFIG, SEARCHER};

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Attaching CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

pub fn command_host(port: u16) -> HskResult<()> {
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
        let rocket = rocket::build()
            .configure(rocket::Config::figment().merge(("port", port)))
            .attach(CORS)
            .mount(
                "/",
                routes![
                    index,
                    search,
                    search_exact,
                    ids,
                    diagnostics,
                    transcript,
                    convert
                ],
            );
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
    let mut timer = Timer::new();
    let page = page.unwrap_or(0);
    let page_results = SEARCHER.search(
        &query,
        context.unwrap_or(CONFIG.context_size()),
        page,
        remove_stop_words,
    );
    timer.print(format!("Searched {query:?}").as_str());
    serde_json::to_string(&page_results).map_err(|err| BadRequest(err.to_string()))
}

#[get("/search_exact?<query>&<page>")]
async fn search_exact(query: String, page: Option<usize>) -> Result<String, BadRequest<String>> {
    let mut timer = Timer::new();
    let page = page.unwrap_or(0);
    let page_results = SEARCHER.search_exact(&query, page);
    timer.print(format!("Searched {query:?}").as_str());
    serde_json::to_string(&page_results).map_err(|err| BadRequest(err.to_string()))
}

#[get("/diagnostics?<query>")]
async fn diagnostics(query: String) -> Result<String, BadRequest<String>> {
    serde_json::to_string(&SEARCHER.diagnose_query(query))
        .map_err(|err| BadRequest(err.to_string()))
}

#[get("/transcript?<path>")]
async fn transcript(path: String) -> Result<String, BadRequest<String>> {
    serde_json::to_string(&SEARCHER.get_transcript_words(path))
        .map_err(|err| BadRequest(err.to_string()))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConversionParameters {
    source: String,
    destination: Option<String>,
    flatten: bool,
}

#[post("/convert", data = "<data>")]
async fn convert(data: String) -> Result<String, BadRequest<String>> {
    let data: ConversionParameters =
        serde_json::from_str(&data).map_err(|err| BadRequest(err.to_string()))?;
    dbg!(&data);
    Ok(serde_json::to_string_pretty(&data).map_err(|err| BadRequest(err.to_string()))?)
}
