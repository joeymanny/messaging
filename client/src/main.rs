use std::str::FromStr;

use lib::Message;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native("My App", options, Box::new(|cc| Box::new(MyApp::new(cc)))).unwrap();
}
struct MyApp {
    menu: Menu,
    server: Option<std::net::IpAddr>,
    ip_submission: String,
}
#[derive(Clone)]
enum Menu {
    Login {
        username: String,
        password: String,
        login_failure: Option<LoginResult>,
    },
    Contacts,
    Chat {
        user_id: u32,
    },
}
#[derive(Clone, Copy, Debug)]
enum LoginResult {
    NoServer,
    ConnectionTimeout,
    NetworkError,
    SomethingWentWrong,
    BadUser,
    BadPass,
    Success,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_style(egui::style::Style {
            override_text_style: Some(egui::style::TextStyle::Monospace),
            ..egui::style::Style::default()
        });
        MyApp {
            menu: Menu::Login {
                username: String::new(),
                password: String::new(),
                login_failure: None,
            },
            server: None,
            ip_submission: String::default(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("server panel").show(ctx, |ui| {
            ui.label(format!(
                "current server: {}",
                if let Some(v) = self.server {
                    v.to_string()
                } else {
                    "None".to_owned()
                }
            ));
            if ui
                .text_edit_singleline(&mut self.ip_submission)
                .lost_focus()
            {
                self.server = self.ip_submission.parse().ok();
                self.ip_submission.clear();
            };
        });

        match self.menu {
            Menu::Login { .. } => login_menu(self, ctx, frame),
            Menu::Contacts => contacts_menu(self, ctx, frame),
            Menu::Chat { user_id } => chat_menu(self, ctx, frame, user_id),
        }
    }
}
fn login_menu(app_state: &mut MyApp, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    let (username, password, login_failure) = match app_state.menu {
        Menu::Login {
            ref mut username,
            ref mut password,
            ref mut login_failure,
        } => (username, password, login_failure),
        _ => panic!("this is unreachable"),
    };
    let mut login_now = false;
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.label("Welcome to login");
            egui::TextEdit::singleline(username)
                .hint_text("username")
                .show(ui);
            egui::TextEdit::singleline(password)
                .password(true)
                .hint_text("password")
                .show(ui);
            login_now = ui.button("log me in scotty").clicked();
            ui.label(format!("login error: {:?}", login_failure));
        });
    });
    if login_now {
        let result = handle_login(app_state.server, username, password);
        if let LoginResult::Success = result {
            app_state.menu = Menu::Contacts;
        } else {
            *login_failure = Some(result);
        }
    }
}
fn handle_login(
    ip: Option<std::net::IpAddr>,
    username: &mut String,
    password: &mut String,
) -> LoginResult {
    let ip = match ip {
        Some(v) => v,
        None => return LoginResult::NoServer
    };
    let userhash = lib::get_hash(username);
    let passhash = lib::get_hash(password);
    username.clear();
    password.clear();
    let mut stream = match std::net::TcpStream::connect_timeout(&std::net::SocketAddr::new(ip, lib::PORT), std::time::Duration::from_secs(1)){
        Ok(v) => v,
        Err(_e) => return LoginResult::ConnectionTimeout
    };
    lib::send_message(
        &mut stream,
        &lib::Message::LoginRequest { username: userhash , password: passhash}
    ).unwrap();
    let response = match lib::get_message(&mut stream){
        Ok(m) => m,
        Err(_e) => return LoginResult::NetworkError,
    };
    match response{
        Message::LoginReply(status) => {
            match status{
                lib::LoginStatus::Accepted => LoginResult::Success,
                lib::LoginStatus::BadPass => LoginResult::BadPass,
                lib::LoginStatus::BadUser => LoginResult::BadUser,
            }
        },
        _ => LoginResult::SomethingWentWrong,

    }
}
fn contacts_menu(app_state: &mut MyApp, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            if ui.button("log out").clicked() {
                app_state.menu = Menu::Login {
                    username: String::new(),
                    password: String::new(),
                    login_failure: None,
                }
            }
        });
        for i in 0..10 {
            if ui.button(i.to_string()).clicked() {
                app_state.menu = Menu::Chat { user_id: i }
            };
        }
    });
}
fn chat_menu(app_state: &mut MyApp, ctx: &egui::Context, _frame: &mut eframe::Frame, chat_id: u32) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if ui.button("back").clicked() {
            app_state.menu = Menu::Contacts;
        }
        ui.vertical_centered(|ui| {
            ui.label(format!("this is chat #{}!", chat_id));
        })
    });
}
