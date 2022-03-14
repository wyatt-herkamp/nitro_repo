use actix_web::{web, HttpServer};
use std::{io, thread, time::Duration};

use crate::api_response::{APIResponse, SiteResponse};

use crate::error::response::mismatching_passwords;
use crate::{installed};
use crate::database::{DbPool, init, init_single_connection};
use actix_web::{post, HttpRequest};

use log::{error, info, trace};
use serde::{Deserialize, Serialize};

use crate::settings::utils::quick_add;
use crate::system::action::add_new_user;

use crate::system::models::{User, UserPermissions};


use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error};
use std::fmt::{Display, Formatter};
use std::fs::{File, OpenOptions};
use std::io::{Stdout, Write};
use diesel::MysqlConnection;
use diesel::types::IsNull::No;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};


use unicode_width::UnicodeWidthStr;
use crate::system::utils::hash;
use crate::utils::get_current_time;

//mysql://newuser:"password"@127.0.0.1/nitro_repo
#[derive(Serialize, Deserialize, Clone, Debug)]
struct DatabaseStage {
    pub user: Option<String>,
    pub password: Option<String>,
    pub host: Option<String>,
    pub database: Option<String>,
}

impl Display for DatabaseStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let my_db = self.clone();
        write!(f, "mysql://{}:{}@{}/{}", my_db.user.unwrap(), my_db.password.unwrap(), my_db.host.unwrap(), my_db.database.unwrap())
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

impl From<UserStage> for User {
    fn from(value: UserStage) -> Self {
        User {
            id: 0,
            name: value.name.unwrap_or_default(),
            username: value.username.unwrap_or_default(),
            email: value.email.unwrap_or_default(),
            password: hash(value.password.unwrap_or_default()).unwrap(),
            permissions: UserPermissions { admin: true, deployer: true },
            created: get_current_time(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OtherStage {
    storage_location: Option<String>,
    log_location: Option<String>,
    address: Option<String>,
    max_upload: Option<String>,
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
    connection: Option<MysqlConnection>,

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
                storage_location: None,
                log_location: None,
                address: None,
                max_upload: None,
            },
        }
    }
}


fn run_app(mut terminal: Terminal<CrosstermBackend<Stdout>>, mut app: App) -> io::Result<Option<App>> {
    loop {
        if app.stage >= 3 {
            close(terminal);
            return Ok(Some(app));
        }
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code
            {
                KeyCode::Enter => {
                    let value = if app.input.is_empty() {
                        get_next_default(&app)
                    } else { Some(app.input.clone()) };
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
                                let result = init_single_connection(&string);
                                match result {
                                    Ok(db) => {
                                        app.connection = Some(db);
                                        app.stage = 1;
                                    }
                                    Err(error) => {
                                        error!("Unable to Connect to Database {} :{}",string,error);
                                        app.database_stage = DatabaseStage {
                                            user: None,
                                            password: None,
                                            host: None,
                                            database: None,
                                        }
                                    }
                                }
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
                                if let Err(error) = add_new_user(&stage.into(), connection) {
                                    error!("Unable to Create user, {}",error);
                                    std::process::exit(1);
                                }

                                app.stage = 2;
                            }
                        }
                        2 => {
                            if app.other_stage.address.is_none() {
                                app.other_stage.address = value;
                            } else if app.other_stage.storage_location.is_none() {
                                app.other_stage.storage_location = value;
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

    Ok(None)
}

fn get_next_default(app: &App) -> Option<String> {
    return match app.stage {
        0 => {
            if app.database_stage.user.is_none() {
                None
            } else if app.database_stage.password.is_none() {
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
            } else if app.other_stage.log_location.is_none() {
                Some("./".to_string())
            } else if app.other_stage.storage_location.is_none() {
                Some("./".to_string())
            } else if app.other_stage.max_upload.is_none() {
                Some("1024".to_string())
            } else {
                None
            }
        }
        _ => {
            None
        }
    };
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
            } else if app.other_stage.storage_location.is_none() {
                "Storage Location".to_string()
            } else if app.other_stage.log_location.is_none() {
                "Log Location".to_string()
            } else if app.other_stage.max_upload.is_none() {
                "Max Upload".to_string()
            } else {
                "Confirm".to_string()
            }
        }
        _ => {
            "".to_string()
        }
    }
}

fn create_line(key: &str, value: &String, messages: &mut Vec<ListItem>) {
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

    create_line("Database Username", app.database_stage.user.as_ref().unwrap_or(&"".to_string()), &mut messages);
    create_line("Database Password", app.database_stage.password.as_ref().unwrap_or(&"".to_string()), &mut messages);
    create_line("Database Host", app.database_stage.host.as_ref().unwrap_or(&"".to_string()), &mut messages);
    create_line("Database Database", app.database_stage.database.as_ref().unwrap_or(&"".to_string()), &mut messages);
    let content = vec![Spans::from(Span::raw("______________User_______________"))];
    messages.push(ListItem::new(content));
    create_line("User Name", app.user_stage.name.as_ref().unwrap_or(&"".to_string()), &mut messages);
    create_line("User Username", app.user_stage.username.as_ref().unwrap_or(&"".to_string()), &mut messages);
    create_line("User Email", app.user_stage.email.as_ref().unwrap_or(&"".to_string()), &mut messages);
    create_line("User Password", app.user_stage.password.as_ref().unwrap_or(&"".to_string()), &mut messages);
    let content = vec![Spans::from(Span::raw("______________Other_______________"))];
    messages.push(ListItem::new(content));
    create_line("Bind Address", app.other_stage.address.as_ref().unwrap_or(&"".to_string()), &mut messages);
    create_line("Log Location", app.other_stage.log_location.as_ref().unwrap_or(&"".to_string()), &mut messages);
    create_line("Storage Location", app.other_stage.storage_location.as_ref().unwrap_or(&"".to_string()), &mut messages);
    create_line("Max Upload", app.other_stage.max_upload.as_ref().unwrap_or(&"".to_string()), &mut messages);


    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Data"));
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
            Span::styled(format!("{}[{}]", string, get_next_default(app).unwrap_or_default()), Style::default().add_modifier(Modifier::BOLD)),
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

pub fn load_installer() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::default();
    let app = run_app(terminal, app).unwrap();
    if app.is_none() {
        println!("Failure to Install");
        return Ok(());
    }
    let app = app.unwrap();
    save_env(&app).unwrap();

    if app.connection.is_none() {
        println!("Failure to Install");
        return Ok(());
    }
    let connection = app.connection.as_ref().unwrap();

    quick_add("installed", "true".to_string(), connection).unwrap();
    quick_add(
        "version",
        env!("CARGO_PKG_VERSION").to_string(),
        connection,
    ).unwrap();
    info!("Installation Complete");
    println!("Installation Complete!");
    Ok(())
}

fn close(mut terminal: Terminal<CrosstermBackend<Stdout>>) {
    disable_raw_mode().unwrap();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).unwrap();
    terminal.show_cursor().unwrap();
}

fn save_env(app: &App) -> anyhow::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(".env")?;
    let my_db = app.database_stage.clone();
    writeln!(&mut file, "DATABASE_URL=mysql://{}:\"{}\"@{}/{}", my_db.user.unwrap(), my_db.password.unwrap(), my_db.host.unwrap(), my_db.database.unwrap())?;
    writeln!(&mut file, "STORAGE_LOCATION={}", &app.other_stage.storage_location.as_ref().unwrap())?;
    writeln!(&mut file, "LOGG_LOCATION={}", &app.other_stage.log_location.as_ref().unwrap())?;
    writeln!(&mut file, "ADDRESS={}", &app.other_stage.address.as_ref().unwrap())?;I
    writeln!(&mut file, "MODE=RELEASE")?;


    Ok(())
}
