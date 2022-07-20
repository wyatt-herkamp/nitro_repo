pub mod utils;

use anyhow::Error;
use std::env::current_dir;
use std::fmt::{Display, Formatter};
use std::fs::{create_dir_all, OpenOptions};
use std::io;
use std::io::{Stdout, Write};
use std::path::PathBuf;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use log::{error, info, trace};
use sea_orm::ActiveValue::Set;
use sea_orm::ConnectionTrait;
use sea_orm::{DatabaseConnection, EntityTrait, Schema};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

use crate::settings::models::{
    Application, Database, EmailSetting, Mode, MysqlSettings, SecuritySettings, SiteSetting,
};
use crate::system::permissions::UserPermissions;
use crate::system::user::UserEntity;
use crate::system::{hash, user};
use crate::utils::get_current_time;
use crate::{authentication, GeneralSettings};

#[derive(Error, Debug)]
pub enum InstallError {
    #[error("Unable to Install Nitro Repo. {0}")]
    InstallError(String),
    #[error("IO Error. Installer. {0}")]
    IOError(std::io::Error),
}

impl From<&str> for InstallError {
    fn from(error: &str) -> Self {
        InstallError::InstallError(error.to_string())
    }
}

impl From<std::io::Error> for InstallError {
    fn from(error: std::io::Error) -> Self {
        InstallError::IOError(error)
    }
}

impl std::convert::From<sea_orm::DbErr> for InstallError {
    fn from(error: sea_orm::DbErr) -> Self {
        InstallError::InstallError(error.to_string())
    }
}

//mysql://newuser:"password"@127.0.0.1/nitro_repo
#[derive(Serialize, Deserialize, Clone, Debug)]
struct DatabaseStage {
    pub user: Option<String>,
    pub password: Option<String>,
    pub host: Option<String>,
    pub database: Option<String>,
}

impl From<DatabaseStage> for Database {
    fn from(db: DatabaseStage) -> Self {
        let mysql_settings = MysqlSettings {
            user: db.user.unwrap(),
            password: db.password.unwrap(),
            host: db.host.unwrap(),
            database: db.database.unwrap(),
        };
        crate::settings::models::Database::Mysql(mysql_settings)
    }
}

impl Display for DatabaseStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let my_db = self.clone();
        write!(
            f,
            "mysql://{}:{}@{}/{}",
            my_db.user.unwrap(),
            my_db.password.unwrap(),
            my_db.host.unwrap(),
            my_db.database.unwrap()
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserStage {
    pub name: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub password_two: Option<String>,
}

impl From<UserStage> for user::database::ActiveModel {
    fn from(value: UserStage) -> Self {
        user::database::ActiveModel {
            id: Default::default(),
            name: Set(value.name.unwrap_or_default()),
            username: Set(value.username.unwrap_or_default()),
            email: Set(value.email.unwrap_or_default()),
            password: Set(hash(value.password.unwrap_or_default()).unwrap()),
            permissions: Set(UserPermissions {
                disabled: false,
                admin: true,
                user_manager: true,
                repository_manager: true,
                deployer: None,
                viewer: None,
            }),
            created: Set(get_current_time()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OtherStage {
    log_location: Option<String>,
    address: Option<String>,
    app_url: Option<String>,
    max_upload: Option<String>,
}

impl From<OtherStage> for Application {
    fn from(other: OtherStage) -> Self {
        let storages = current_dir().unwrap().join("storages");
        Self {
            log: other.log_location.unwrap(),
            address: other.address.unwrap(),
            app_url: other.app_url.unwrap(),
            max_upload: other.max_upload.unwrap().parse().unwrap(),
            mode: Mode::Release,
            storage_location: storages,
            ssl_private_key: None,
            ssl_cert_key: None,
        }
    }
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// Current input mode
    stage: u8,
    /// History of recorded messages
    database_stage: DatabaseStage,
    user_stage: UserStage,
    other_stage: OtherStage,
    connection: Option<DatabaseConnection>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            stage: 0,
            connection: None,
            database_stage: DatabaseStage {
                user: None,
                password: None,
                host: None,
                database: None,
            },
            user_stage: UserStage {
                name: None,
                username: None,
                email: None,
                password: None,
                password_two: None,
            },
            other_stage: OtherStage {
                log_location: None,
                address: None,
                app_url: None,
                max_upload: None,
            },
        }
    }
}
#[allow(unused_mut)]
async fn run_app(
    mut terminal: Terminal<CrosstermBackend<Stdout>>,
    mut app: App,
) -> Result<App, InstallError> {
    loop {
        if app.stage >= 3 {
            close(terminal);
            return Ok(app);
        }
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => {
                    let value = if app.input.is_empty() {
                        get_next_default(&app)
                    } else {
                        Some(app.input.clone())
                    };
                    app.input.clear();
                    match app.stage {
                        0 => {
                            if app.database_stage.user.is_none() {
                                app.database_stage.user = value;
                            } else if app.database_stage.password.is_none() {
                                app.database_stage.password = value;
                            } else if app.database_stage.host.is_none() {
                                app.database_stage.host = value;
                            } else if app.database_stage.database.is_none() {
                                app.database_stage.database = value;
                            } else {
                                let string = app.database_stage.to_string();
                                trace!("Database String: {}", &string);
                                let mut database_conn = sea_orm::Database::connect(string).await?;
                                let schema = Schema::new(database_conn.get_database_backend());
                                let users = schema.create_table_from_entity(UserEntity);
                                database_conn
                                    .execute(database_conn.get_database_backend().build(&users))
                                    .await?;
                                let tokens = schema.create_table_from_entity(
                                    authentication::auth_token::AuthTokenEntity,
                                );
                                database_conn
                                    .execute(database_conn.get_database_backend().build(&tokens))
                                    .await?;

                                app.connection = Some(database_conn);
                                app.stage = 1;
                            }
                        }
                        1 => {
                            if app.user_stage.name.is_none() {
                                app.user_stage.name = value;
                            } else if app.user_stage.username.is_none() {
                                app.user_stage.username = value;
                            } else if app.user_stage.email.is_none() {
                                app.user_stage.email = value;
                            } else if app.user_stage.password.is_none() {
                                app.user_stage.password = value;
                            } else {
                                let connection = app.connection.as_ref().unwrap();
                                let stage = app.user_stage.clone();

                                //TODO dont kill program on failure to create user
                                let user: user::database::ActiveModel = stage.into();
                                UserEntity::insert(user).exec(connection).await?;
                                app.stage = 2;
                            }
                        }
                        2 => {
                            if app.other_stage.address.is_none() {
                                app.other_stage.address = value;
                            } else if app.other_stage.app_url.is_none() {
                                app.other_stage.app_url = value;
                            } else if app.other_stage.log_location.is_none() {
                                app.other_stage.log_location = value;
                            } else if app.other_stage.max_upload.is_none() {
                                app.other_stage.max_upload = value;
                            } else {
                                app.stage = 3;
                            }
                        }
                        _ => {}
                    }
                }
                KeyCode::Char(c) => {
                    app.input.push(c);
                }
                KeyCode::Backspace => {
                    app.input.pop();
                }
                KeyCode::Esc => {
                    break;
                }
                _ => {}
            }
        }
    }
    close(terminal);

    Err("Unknown Error. Final Catch!".into())
}

fn get_next_default(app: &App) -> Option<String> {
    match app.stage {
        0 => {
            if app.database_stage.user.is_none() || app.database_stage.password.is_none() {
                None
            } else if app.database_stage.host.is_none() {
                Some("127.0.0.1".to_string())
            } else if app.database_stage.database.is_none() {
                Some("nitro_repo".to_string())
            } else {
                None
            }
        }
        2 => {
            if app.other_stage.address.is_none() {
                Some("0.0.0.0:6742".to_string())
            } else if app.other_stage.app_url.is_none() {
                Some("http://127.0.0.1:6742".to_string())
            } else if app.other_stage.log_location.is_none() {
                Some("./".to_string())
            } else if app.other_stage.max_upload.is_none() {
                Some("1024".to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_next_step(app: &App) -> String {
    match app.stage {
        0 => {
            if app.database_stage.user.is_none() {
                "Database Username".to_string()
            } else if app.database_stage.password.is_none() {
                "Database Password".to_string()
            } else if app.database_stage.host.is_none() {
                "Database Host".to_string()
            } else if app.database_stage.database.is_none() {
                "Database Database".to_string()
            } else {
                "Confirm".to_string()
            }
        }
        1 => {
            if app.user_stage.name.is_none() {
                "User Name".to_string()
            } else if app.user_stage.username.is_none() {
                "User Username".to_string()
            } else if app.user_stage.email.is_none() {
                "User Email".to_string()
            } else if app.user_stage.password.is_none() {
                "User Password".to_string()
            } else {
                "Confirm".to_string()
            }
        }
        2 => {
            if app.other_stage.address.is_none() {
                "Bind Address".to_string()
            } else if app.other_stage.app_url.is_none() {
                "App URL".to_string()
            } else if app.other_stage.log_location.is_none() {
                "Log Location".to_string()
            } else if app.other_stage.max_upload.is_none() {
                "Max Upload".to_string()
            } else {
                "Confirm".to_string()
            }
        }
        _ => "".to_string(),
    }
}

fn create_line(key: &str, value: &str, messages: &mut Vec<ListItem>) {
    let content = vec![Spans::from(Span::raw(format!("{key}: {value}")))];
    messages.push(ListItem::new(content))
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Min(5),
                Constraint::Length(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());
    let mut messages: Vec<ListItem> = Vec::new();

    create_line(
        "Database Username",
        app.database_stage.user.as_ref().unwrap_or(&"".to_string()),
        &mut messages,
    );
    create_line(
        "Database Password",
        app.database_stage
            .password
            .as_ref()
            .unwrap_or(&"".to_string()),
        &mut messages,
    );
    create_line(
        "Database Host",
        app.database_stage.host.as_ref().unwrap_or(&"".to_string()),
        &mut messages,
    );
    create_line(
        "Database Database",
        app.database_stage
            .database
            .as_ref()
            .unwrap_or(&"".to_string()),
        &mut messages,
    );
    let content = vec![Spans::from(Span::raw("______________User_______________"))];
    messages.push(ListItem::new(content));
    create_line(
        "User Name",
        app.user_stage.name.as_ref().unwrap_or(&"".to_string()),
        &mut messages,
    );
    create_line(
        "User Username",
        app.user_stage.username.as_ref().unwrap_or(&"".to_string()),
        &mut messages,
    );
    create_line(
        "User Email",
        app.user_stage.email.as_ref().unwrap_or(&"".to_string()),
        &mut messages,
    );
    create_line(
        "User Password",
        app.user_stage.password.as_ref().unwrap_or(&"".to_string()),
        &mut messages,
    );
    let content = vec![Spans::from(Span::raw("______________Other_______________"))];
    messages.push(ListItem::new(content));
    create_line(
        "Bind Address",
        app.other_stage.address.as_ref().unwrap_or(&"".to_string()),
        &mut messages,
    );
    create_line(
        "App URL",
        app.other_stage.app_url.as_ref().unwrap_or(&"".to_string()),
        &mut messages,
    );
    create_line(
        "Log Location",
        app.other_stage
            .log_location
            .as_ref()
            .unwrap_or(&"".to_string()),
        &mut messages,
    );

    create_line(
        "Max Upload",
        app.other_stage
            .max_upload
            .as_ref()
            .unwrap_or(&"".to_string()),
        &mut messages,
    );

    let messages = List::new(messages).block(Block::default().borders(Borders::ALL).title("Data"));
    f.render_widget(messages, chunks[0]);

    let string = get_next_step(app);

    let msg = if string.eq("Confirm") {
        vec![
            Span::raw("Please Enter to Confirm "),
            Span::styled(". ESC", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!(" To Restart. {}", app.stage)),
        ]
    } else {
        vec![
            Span::raw("Please Enter "),
            Span::styled(
                format!("{}[{}]", string, get_next_default(app).unwrap_or_default()),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(". Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!(" to record the data. {}", app.stage)),
        ]
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(Style::default());
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[1]);

    let input = Paragraph::new(app.input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[2]);
    f.set_cursor(
        // Put cursor past the end of the input text
        chunks[2].x + app.input.width() as u16 + 1,
        // Move one line down, from the border to the input line
        chunks[2].y + 1,
    )
}

pub async fn load_installer(working_dir: PathBuf) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    let app = run_app(terminal, App::default()).await?;

    install_data(working_dir, app)
}

fn install_data(working_dir: PathBuf, app: App) -> Result<(), Error> {
    let general = GeneralSettings {
        database: app.database_stage.into(),
        application: Application::from(app.other_stage),
        internal: Default::default(),
        session: Default::default(),
    };
    let configs = working_dir.join("cfg");
    create_dir_all(&configs)?;

    let other = toml::to_string_pretty(&general)?;
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(working_dir.join("nitro_repo.toml"))?;
    file.write_all(other.as_bytes())?;

    let security = toml::to_string_pretty(&SecuritySettings::default())?;
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(configs.join("security.toml"))?;
    file.write_all(security.as_bytes())?;

    let email = toml::to_string_pretty(&EmailSetting::default())?;
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(configs.join("email.toml"))?;
    file.write_all(email.as_bytes())?;
    let site = toml::to_string_pretty(&SiteSetting::default())?;
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(configs.join("site.toml"))?;
    file.write_all(site.as_bytes())?;
    info!("Installation Complete");
    Ok(())
}

fn close(mut terminal: Terminal<CrosstermBackend<Stdout>>) {
    disable_raw_mode().unwrap();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .unwrap();
    terminal.show_cursor().unwrap();
}
