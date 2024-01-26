
use reqwest::{blocking::Client, header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE}, Url};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io;
use std::ptr::null;
use base64::encode;
use reqwest::header::HeaderMap;
use serde_json::{json, Number};
use crate::services::Jira::{ISSUE_LOGWORK_URL_PART, ISSUE_URL_PART};

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraLoginResponse {
    pub session: JiraSession,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraSession {
    pub name: String,
    pub value: String,
}
#[derive( Clone)]
pub struct JiraClient {
     base_url: String,
    credentials:String,
     client: Client,
     current_task_id: String
}


#[derive(Debug, Deserialize, Serialize)]
pub struct Issue {
    fields: IssueFields,
}

#[derive(Debug, Deserialize, Serialize)]
struct IssueFields {
    worklog: Worklog,
    summary: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Worklog {
    worklogs: Vec<WorklogEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WorklogEntry {
    #[serde(rename = "self")]
    self_link: Option<String>,
    author: User,
    updateAuthor: User,
    created: String,
    updated: String,
    started: String,
    timeSpent: String,
    timeSpentSeconds: i32,
    id: String,
    issueId: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct User {
    self_link: Option<String>,
    accountId: String,
    emailAddress: String,
    avatarUrls: AvatarUrls,
    displayName: String,
    active: bool,
    timeZone: String,
    accountType: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AvatarUrls {
    #[serde(rename = "48x48")]
    size_48x48: String,
    #[serde(rename = "24x24")]
    size_24x24: String,
    #[serde(rename = "16x16")]
    size_16x16: String,
    #[serde(rename = "32x32")]
    size_32x32: String,
}
impl JiraClient {

    pub fn new(current_task_url: &str, username: &str, password: &str) -> Result<JiraClient, io::Error> {
        let client = Client::new();

        let parsed_url = Url::parse(current_task_url).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidInput, format!("Failed to parse URL: {}", e))
        })?;

        let base_url = format!("{}://{}", parsed_url.scheme(), parsed_url.host_str().unwrap_or(""));

        let credentials = encode(format!("{}:{}", username, password));

        let parts: Vec<&str> = current_task_url.split('/').collect();

        // Get the last part
        let task_id = parts.last().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "TaskId not recognized"))?.to_string();

        Ok(JiraClient {
            base_url,
            credentials,
            client,
            current_task_id: task_id,
        })
    }
    pub  fn post_worklog (
        &self,
        worklog_text: String,
        worklog_seconds: i64,
        started: String,
    ) -> Result<String, String> {

        let url = format!("{}{}{}/worklog", self.base_url, ISSUE_URL_PART,self.current_task_id);

        let body_data = json!({
        "comment": {
            "content": [{
                "content": [{
                    "text": worklog_text,
                    "type": "text"
                }],
                "type": "paragraph"
            }],
            "type": "doc",
            "version": 1
        },
        "started": started,
        "timeSpentSeconds": worklog_seconds
    });
        let auth_header_value = format!("Basic {}", self.credentials);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_header_value).map_err(|e| format!("{}", e))?);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let response = self.client
            .post(&url)
            .json(&body_data)
            .headers(headers)
            .send()
            .map_err(|e| format!("{}", e))?;

                if response.status().is_success() {
                    println!("Response: {} {}", response.status(), response.text().unwrap());
                    Ok("OK".into())
                } else {
                    let error_message = response.text().map_err(|e| format!("Error reading error response body: {}", e))?;
                    Err(format!("Error: {}", error_message))

                }
    }
    pub fn getIssueData(&self, id: String) ->Result<Issue, String> {
        let get_issue_data_url = if(id.is_empty()){
             format!("{}{}", self.base_url, ISSUE_URL_PART)+&self.current_task_id
        }
        else {
            format!("{}{}", self.base_url, ISSUE_URL_PART) + &id
        };

        let auth_header_value = format!("Basic {}", self.credentials);

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_header_value).map_err(|e| format!("{}", e))?);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));


        let response = self.client
            .get(&get_issue_data_url)
            .headers(headers)
            .send()
            .map_err(|e| format!("{}", e))?;

        if response.status().is_success() {
            let issue: Result<Issue, _> = serde_json::from_str(&response.text().unwrap().as_str());

            if let Ok(issue) = issue {


                // let login_response: JiraLoginResponse = response.json().map_err(|e| format!("{}", e))?;
                return Ok(issue);
            } else if let Err(e) = issue {
                println!("Error deserializing JSON: {}", e);
                // Handle the error gracefully, e.g., return an error or log it.
                return Err(e.to_string());
            }

        } else {

            println!("Login failed. Status code: {}, text:{}", response.status(),response.text().unwrap().as_str());

        }
        Err("Login failed".into())
    }
}