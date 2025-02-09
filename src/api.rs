use std::fmt;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use worker::console_log;

const API_ENDPOINT: &str = "https://api.cloudflare.com/client/v4";

// The Cloudflare API docs label a *lot* of response object fields as optional that seem like they
// shouldn't be? It's unclear in many cases what it would mean for that field to be missing, except
// maybe in the case of an error response. In some of these cases, we're going to assume that
// they'll be present and let serde return an error if they're not.

#[derive(Debug)]
pub struct ApiClient {
    account_id: String,
    api_token: String,
    client: Client,
}

impl ApiClient {
    pub fn new(api_token: String, account_id: String) -> Self {
        Self {
            account_id,
            api_token,
            client: Client::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct DbExportBookmark(String);

impl fmt::Display for DbExportBookmark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Deserialize)]
struct EnvelopeError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct Envelope<T> {
    result: T,
    success: bool,
    errors: Vec<EnvelopeError>,
}

impl<T> Envelope<T> {
    fn unwrap_result(self) -> anyhow::Result<T> {
        if !self.success {
            return Err(anyhow::anyhow!(
                "Request failed with errors: {:?}",
                self.errors
            ));
        }

        Ok(self.result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
enum DbExportStatus {
    #[serde(rename = "complete")]
    Complete,

    #[serde(rename = "error")]
    Error,
}

#[derive(Debug, Serialize)]
struct DbExportBody {
    output_format: String,
}

impl Default for DbExportBody {
    fn default() -> Self {
        Self {
            output_format: "polling".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct DbExportResponseResult {
    signed_url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct DbExportResponse {
    success: bool,
    status: Option<DbExportStatus>,
    error: Option<String>,
    at_bookmark: Option<DbExportBookmark>,
    result: Option<DbExportResponseResult>,
}

impl DbExportResponse {
    fn unwrap_result(self) -> anyhow::Result<Option<DbExportResponseResult>> {
        if !self.success {
            anyhow::bail!("Request failed with error: {:?}", self.error);
        }

        match self.status {
            Some(DbExportStatus::Complete) => Ok(self.result),
            Some(DbExportStatus::Error) => {
                anyhow::bail!("Request failed with error: {:?}", self.error)
            }
            None => Ok(None),
        }
    }
}

#[derive(Debug)]
pub struct DbExport {
    bookmark: DbExportBookmark,
    signed_url: Option<String>,
}

impl ApiClient {
    // TODO: This method only *starts*, the DB export. Per the Cloudflare API docs, it needs to be
    // polled periodically to complete the DB export, otherwise it will automatically cancel. From
    // the docs:
    //
    //   Note: this process may take some time for larger DBs, during which your D1 will be
    //   unavailable to serve queries. To avoid blocking your DB unnecessarily, an in-progress
    //   export must be continually polled or will automatically cancel.
    //
    // In practice, the Ace Archive DB seems to be small enough to always export successfully on
    // the first call to this endpoint. We *should* implement the polling behavior anyways to
    // ensure that it always succeeds going forward, however I'm not able to test this behavior of
    // the Cloudflare API without a large enough DB, and the semantics of this API endpoint aren't
    // clear enough to me just from reading the docs to implement this without being able to play
    // around with the API endpoint.
    pub async fn start_db_export(&self, db_id: &str) -> anyhow::Result<DbExport> {
        let account_id = &self.account_id;

        let resp = self
            .client
            .post(format!(
                "{API_ENDPOINT}/accounts/{account_id}/d1/database/{db_id}/export",
            ))
            .body(serde_json::to_string(&DbExportBody::default())?)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?
            .error_for_status()?;

        let wrapped_resp_body = resp.json::<Envelope<DbExportResponse>>().await?;
        let resp_body = wrapped_resp_body.unwrap_result()?;
        let maybe_result = resp_body.clone().unwrap_result()?;

        console_log!("Started DB export for DB ID: {db_id}");

        if let Some(bookmark) = &resp_body.at_bookmark {
            console_log!("Got DB bookmark: {bookmark}");
        }

        if let Some(signed_url) = maybe_result.as_ref().map(|r| &r.signed_url) {
            console_log!("Got DB signed download URL: {signed_url}");
        }

        Ok(DbExport {
            bookmark: resp_body
                .at_bookmark
                .ok_or_else(|| anyhow::anyhow!("No DB bookmark in response, even though the response did not indicate an error"))?,
            signed_url: maybe_result.map(|r| r.signed_url),
        })
    }
}
