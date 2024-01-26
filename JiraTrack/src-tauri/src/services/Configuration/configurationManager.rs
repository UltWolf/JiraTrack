use std::fs;
use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use base64::encode;
use reqwest::blocking::Client;
use crate::services::Configuration::{PATH_TO_CONFIGURATION_FILE, PATH_TO_CONFIGURATION_FOLDER};
use crate::services::Jira::jiraClient::JiraClient;


#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct Configuration {
    username: String,
    password:String,
    url:String,
    last_task:String,
}

impl Configuration {
    pub fn new(username: &str,
               password:&str,
               url:&str,
               last_task:&str) -> Self {

        Configuration {
            username: username.to_string(),
            password: password.to_string(),
            url:url.to_string(),
            last_task:last_task.to_string()
        }
    }
    pub fn saveConfiguration(&self){
        let mut base_path = PATH_TO_CONFIGURATION_FOLDER.to_owned();
         base_path.push_str("/");
        base_path.push_str(PATH_TO_CONFIGURATION_FILE);
        let serialized_data = bincode::serialize(&self).expect("Serialization failed");

        let path = Path::new(PATH_TO_CONFIGURATION_FOLDER);
        if !path.exists() {
            fs::create_dir(PATH_TO_CONFIGURATION_FOLDER);
        }
        let mut file = File::create(base_path).expect("Unable to create file");
        file.write_all(&serialized_data).expect("Unable to write to file");
    }
    pub fn loadConfiguration() -> Configuration {
        let base_path = PathBuf::from(PATH_TO_CONFIGURATION_FOLDER);
        let combined_path = base_path.join(PATH_TO_CONFIGURATION_FILE);
        let path = Path::new(&combined_path);

        if path.exists() {
            let mut file = File::open(combined_path).expect("Unable to open file");
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).expect("Unable to read from file");
            let deserialized_data: Configuration = bincode::deserialize(&buffer).expect("Deserialization failed");
            return deserialized_data;
        }
        return Configuration::new("","","","");
    }
}