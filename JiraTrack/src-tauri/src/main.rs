// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod services;

use std::error::Error;
use std::future::Future;
use std::sync::Mutex;
use bincode::config;
use lazy_static::lazy_static;
use serde_json::json;
use services::Jira::jiraClient::{JiraClient, JiraLoginResponse};
use tauri::{SystemTray, SystemTrayMenu, SystemTrayEvent, CustomMenuItem, SystemTrayMenuItem, Window, generate_context, Size, PhysicalSize};
use tauri::async_runtime::set;
use tauri::Manager;
use crate::services::Configuration::configurationManager::Configuration;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
lazy_static! {
    static ref CONFIGURATION: Mutex<Option<Configuration>> = Mutex::new(None);
    static ref JIRA_CLIENT: Mutex<Option<JiraClient>> = Mutex::new(None);
}
#[tauri::command]
fn login(handle: tauri::AppHandle, username:String, url:String, password:String)->Result<(), String> {
    let jira_client = JiraClient::new(
        url.as_str(),
        username.as_str(),
        password.as_str());
    // match  {
    //     Ok(response) => {
    //         let docs_window = tauri::WindowBuilder::new(
    //             &handle,
    //             "main",
    //             tauri::WindowUrl::App("configuration.html".parse().unwrap()),
    //         )
    //             .build()
    //             .unwrap();
    //         return Ok(());
    //     }
    //     Err(err) => {
    //         eprintln!("Error: {}", err)
    //         // Handle the error appropriately
    //     }
    // }
    Err("err".into())
}
#[tauri::command]
fn get_configuration(window: tauri::Window)->Result<String, String>{
   let result = Configuration::loadConfiguration();

    let json_string = serde_json::to_string(&result).unwrap();
    set_global_configuration(result);
    return Ok(json_string.into())
}

#[tauri::command]
fn set_configuration( handle: tauri::AppHandle,
                      username: &str,
                      password: &str,
                      url:&str,
                       )
{
    let configuration = Configuration::new(username,password,url, "");
    configuration.saveConfiguration();
    set_global_configuration(configuration);

}
#[tauri::command]
fn get_task( handle: tauri::AppHandle,
                      username: &str,
                      password: &str,
                      url:&str,
)->Result<String, String>
{

    let jira_client =
        match JiraClient::new(
        url,
        username,
        password){
            Ok(jira_client) => {
                // Assuming you have a method like `getIssueData` in your `JiraClient` struct
                let result = jira_client.getIssueData("".to_string());
                let json_string = serde_json::to_string(&result).unwrap();
                set_global_client(jira_client);
                println!("{}",json_string);
                return Ok(json_string.into());
            }
            Err(err) => {
               return  Err(err.to_string());
            }
        };
}
#[tauri::command]
 fn worklog_task(
             worklog_text: String,
             worklog_sec: i64,
             started:String,
)->Result<String, String>
{

    match  JIRA_CLIENT.lock().unwrap().clone().unwrap().post_worklog(worklog_text, worklog_sec, started)
    {
        Ok(result) => {
          return Ok(result);
        }
        Err(result)=>{
            return Err(result)
        }
    }
}
fn main() { 
  let quit = CustomMenuItem::new("quit".to_string(), "Quit");
  let hide = CustomMenuItem::new("hide".to_string(), "Hide");
  let stop = CustomMenuItem::new("stop".to_string(), "stop");
  let track = CustomMenuItem::new("track".to_string(), "track");
  let tray_menu = SystemTrayMenu::new()
  .add_item(quit)
  .add_native_item(SystemTrayMenuItem::Separator)
  .add_item(hide)
  .add_native_item(SystemTrayMenuItem::Separator)
  .add_item(stop)
  .add_native_item(SystemTrayMenuItem::Separator)
  .add_item(track);
   tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![set_configuration,
                                                 get_configuration,
                                                 get_task,
                                                 login,
                                                  worklog_task
        ])
        .system_tray(SystemTray::new().with_menu(tray_menu))
    .on_system_tray_event(|app, event| match event {
      SystemTrayEvent::LeftClick {
        position: _,
        size: _,
        ..
      } => {
        println!("system tray received a left click");
      }
      SystemTrayEvent::RightClick {
        position: _,
        size: _,
        ..
      } => {
        println!("system tray received a right click");
      }
      SystemTrayEvent::DoubleClick {
        position: _,
        size: _,
        ..
      } => {
        println!("system tray received a double click");
      }
      SystemTrayEvent::MenuItemClick { id, .. } => {
        match id.as_str() {
          "quit" => {
            std::process::exit(0);
          }
          "hide" => {
            let window = app.get_window("main").unwrap();
            window.hide().unwrap();
          }
          "New Content"=>{
              println!("hello")
          },"login"=>{
            println!("hello")
          }
          _ => {}
        }
      }
      _ => {}
    }).on_window_event(|event| match event.event() {
      tauri::WindowEvent::CloseRequested { api, .. } => {
        event.window().hide().unwrap();
        api.prevent_close();
      }
      _ => {}
    })
    .setup(|app| {
      let handle = app.handle(); 
      std::thread::spawn(move || {
        let local_window = tauri::WindowBuilder::new(
          &handle,
          "local",
          tauri::WindowUrl::App("index.html".into())
        ).build().expect("Failed to attach event listener");
          local_window.set_size(Size::Physical(PhysicalSize { width: 1366, height: 768 })).unwrap();
          local_window.listen("configurationUpdated", move |event| {
              // Handle the received event
              println!("received event");
              if let Some(data) = event.payload() {
                  println!("Received data in local window: {}", data);
                  // Do something with the data
              }
          });
          return local_window;
      });
      Ok(())
    })
       .on_window_event(|event| match event.event() {
           tauri::WindowEvent::CloseRequested { api, .. } => {
               event.window().hide().unwrap();
               api.prevent_close();
           }
           _ => {}
       })
       .run(tauri::generate_context!())
       .expect("error while running tauri application");
}
fn set_global_configuration(configuration: Configuration){
    *CONFIGURATION.lock().unwrap() = Some(configuration);
}

fn set_global_client(JiraClient: JiraClient){
    *JIRA_CLIENT.lock().unwrap() = Some(JiraClient);
}

