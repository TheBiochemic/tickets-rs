#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Uncomment for debugging build

use std::{
    sync::{
    Arc, 
    Mutex
    }, 
};

use tickets_rs_adapters::LocalTicketAdapter;

use tickets_rs_core::{
    AppConfig,
    LocalDatabase,
    TicketProvider,
    AdapterType
};

use tickets_rs_ui::{
    UserInterface, 
    UIController, 
    UITheme
};


fn main() {
    let database = {
        let database = match LocalDatabase::open("./app_config.db3".to_string()) {
            Ok(success) => success,
            Err(_) => {
                println!("Failed to read Local SQLite Database, exiting!"); return;
            }
        };
        Arc::new(Mutex::new(database))
    };

    let configuration = Arc::new(Mutex::new(AppConfig::new(database)));
    let ticket_provider = Arc::new(Mutex::new( {
        TicketProvider::new(configuration.clone(), vec![

            AdapterType::new::<LocalTicketAdapter>()

        ])
    }));


    let ui_controller = UIController::new(configuration.clone(), ticket_provider);
    let ui_theme = UITheme::from(configuration);
    UserInterface::launch(ui_controller, ui_theme);

}