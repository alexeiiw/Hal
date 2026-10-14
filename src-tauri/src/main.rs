#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, fs, path::PathBuf, thread, time::Duration};

use serde::{Deserialize, Serialize};
use sysinfo::{ProcessesToUpdate, System};
use tauri::{Emitter, Manager, PhysicalPosition, Position};

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct TelegramConfig {
    token: String,
    chat_id: Option<i64>,
    offset: Option<i64>,
}

#[derive(Deserialize)]
struct TelegramResponse {
    ok: bool,
    result: Vec<TelegramUpdate>,
}

#[derive(Deserialize)]
struct TelegramUpdate {
    update_id: i64,
    message: Option<TelegramMessage>,
}

#[derive(Deserialize)]
struct TelegramMessage {
    chat: TelegramChat,
    text: Option<String>,
}

#[derive(Deserialize)]
struct TelegramChat {
    id: i64,
}

#[derive(Clone, Serialize)]
struct ViewChange {
    view: String,
    enabled: bool,
}

fn telegram_config_path() -> PathBuf {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("HAL")
        .join("telegram.json")
}

fn load_telegram_config(path: &PathBuf) -> Option<TelegramConfig> {
    if !path.exists() {
        fs::create_dir_all(path.parent()?).ok()?;
        fs::write(
            path,
            serde_json::to_string_pretty(&TelegramConfig::default()).ok()?,
        )
        .ok()?;
    }

    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

fn save_telegram_config(path: &PathBuf, config: &TelegramConfig) {
    if let Ok(content) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, content);
    }
}

async fn telegram_send(client: &reqwest::Client, token: &str, chat_id: i64, text: &str) {
    let _ = client
        .post(format!("https://api.telegram.org/bot{token}/sendMessage"))
        .form(&[("chat_id", chat_id.to_string()), ("text", text.to_string())])
        .send()
        .await;
}

async fn handle_telegram_command(
    app: &tauri::AppHandle,
    client: &reqwest::Client,
    config: &mut TelegramConfig,
    message: TelegramMessage,
) {
    let chat_id = message.chat.id;
    let Some(text) = message.text.map(|text| text.trim().to_string()) else {
        return;
    };

    if config.chat_id.is_none() {
        if text == "/pair" {
            config.chat_id = Some(chat_id);
            telegram_send(
                client,
                &config.token,
                chat_id,
                "HAL vinculado. Usa /help para ver los comandos.",
            )
            .await;
        } else {
            telegram_send(
                client,
                &config.token,
                chat_id,
                "Envía /pair para vincular este chat con HAL.",
            )
            .await;
        }
        return;
    }

    if config.chat_id != Some(chat_id) {
        return;
    }

    if text == "/pair" {
        telegram_send(
            client,
            &config.token,
            chat_id,
            "HAL ya está vinculado a este chat.",
        )
        .await;
        return;
    }

    if text == "/off hal" {
        telegram_send(client, &config.token, chat_id, "HAL se está apagando.").await;
        app.exit(0);
        return;
    }

    let response = if text == "/help" || text == "/start" {
        "Comandos:\n/pair\n/inventory\n/mode normal|developer|focus|relax\n/on nombre\n/off nombre\n/off hal\n/phrase texto\n/task texto\n/status"
            .to_string()
    } else if text == "/inventory" {
        "Vistas disponibles:\ncpu, ram, clima, reloj, btc, cara, tarea\nModos: normal, developer, focus, relax\nUsa /on nombre o /off nombre."
            .to_string()
    } else if text == "/status" {
        "HAL vinculado y esperando comandos.".to_string()
    } else if let Some(mode) = text.strip_prefix("/mode ") {
        match mode.trim() {
            "normal" | "developer" | "focus" | "relax" => {
                let _ = app.emit("evento-modo", mode.trim());
                format!("Modo {} activado.", mode.trim())
            }
            _ => "Modo inválido. Usa normal, developer, focus o relax.".to_string(),
        }
    } else if let Some(view) = text.strip_prefix("/on ") {
        set_view(app, view.trim(), true)
    } else if let Some(view) = text.strip_prefix("/off ") {
        set_view(app, view.trim(), false)
    } else if text == "/task" {
        "Usa /task seguido de una tarea de hasta 30 caracteres.".to_string()
    } else if let Some(task) = text.strip_prefix("/task ") {
        let task = task.trim();

        if task == "clear" {
            let _ = app.emit("evento-tarea", Option::<String>::None);
            "Tarea eliminada.".to_string()
        } else if task.is_empty() || task.chars().count() > 30 {
            "La tarea debe tener entre 1 y 30 caracteres.".to_string()
        } else {
            let _ = app.emit("evento-tarea", Some(task.to_string()));
            "Tarea prioritaria actualizada.".to_string()
        }
    } else if let Some(phrase) = text.strip_prefix("/phrase ") {
        let phrase = phrase.trim();

        if phrase == "clear" {
            let _ = app.emit("evento-frase", Option::<String>::None);
            "Frase eliminada.".to_string()
        } else if phrase.is_empty() || phrase.chars().count() > 30 {
            "La frase debe tener entre 1 y 30 caracteres.".to_string()
        } else {
            let _ = app.emit("evento-frase", Some(phrase.to_string()));
            "Frase actualizada.".to_string()
        }
    } else {
        "Comando no reconocido. Usa /help.".to_string()
    };

    telegram_send(client, &config.token, chat_id, &response).await;
}

fn set_view(app: &tauri::AppHandle, view: &str, enabled: bool) -> String {
    let view = match view {
        "ram" => "memory",
        "clima" => "weather",
        "reloj" => "clock",
        "btc" => "bitcoin",
        "cara" => "face",
        "tarea" => "task",
        view => view,
    };

    match view {
        "cpu" | "memory" | "weather" | "clock" | "bitcoin" | "face" | "task" => {
            let _ = app.emit(
                "evento-vista",
                ViewChange {
                    view: view.to_string(),
                    enabled,
                },
            );
            format!(
                "Vista {} {}.",
                view,
                if enabled { "activada" } else { "desactivada" }
            )
        }
        _ => "Vista inválida. Usa cpu, ram, clima, reloj, btc, cara o tarea.".to_string(),
    }
}

async fn run_telegram(app: tauri::AppHandle) {
    let path = telegram_config_path();
    let Some(mut config) = load_telegram_config(&path) else {
        return;
    };

    if config.token.trim().is_empty() {
        return;
    }

    let client = reqwest::Client::new();

    loop {
        let offset = config.offset.unwrap_or(0).to_string();
        let response = client
            .get(format!(
                "https://api.telegram.org/bot{}/getUpdates",
                config.token
            ))
            .query(&[("timeout", "20"), ("offset", offset.as_str())])
            .send()
            .await;

        let Ok(response) = response else {
            tokio::time::sleep(Duration::from_secs(30)).await;
            continue;
        };

        let Ok(updates) = response.json::<TelegramResponse>().await else {
            tokio::time::sleep(Duration::from_secs(30)).await;
            continue;
        };

        if !updates.ok {
            tokio::time::sleep(Duration::from_secs(30)).await;
            continue;
        }

        for update in updates.result {
            config.offset = Some(update.update_id + 1);
            if let Some(message) = update.message {
                handle_telegram_command(&app, &client, &mut config, message).await;
            }
        }
        save_telegram_config(&path, &config);
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").expect("missing main window");
            window
                .set_ignore_cursor_events(true)
                .expect("failed to enable click-through");

            if let Ok(Some(monitor)) = window.current_monitor() {
                let work_area = monitor.work_area();
                let x = work_area.position.x + work_area.size.width as i32 - 184;
                let y = work_area.position.y + work_area.size.height as i32 - 144;
                window
                    .set_position(Position::Physical(PhysicalPosition::new(x, y)))
                    .expect("failed to position HAL");
            }

            let app_handle = app.handle().clone();
            thread::spawn(move || {
                let mut system = System::new_all();
                system.refresh_cpu_all();

                loop {
                    thread::sleep(Duration::from_secs(1));
                    system.refresh_cpu_all();
                    system.refresh_memory();
                    system.refresh_processes(ProcessesToUpdate::All, true);
                    let cpu = system.global_cpu_usage();
                    let memory = if system.total_memory() == 0 {
                        0.0
                    } else {
                        system.used_memory() as f32 / system.total_memory() as f32 * 100.0
                    };
                    let (opencode_active, opencode_cpu) = system
                        .processes()
                        .values()
                        .filter(|process| {
                            let name = process.name().to_string_lossy();
                            name.eq_ignore_ascii_case("opencode")
                                || name.eq_ignore_ascii_case("opencode.exe")
                        })
                        .fold((false, 0.0), |(_, total), process| {
                            (true, total + process.cpu_usage())
                        });

                    if app_handle.emit("evento-cpu", cpu).is_err() {
                        break;
                    }

                    if app_handle.emit("evento-memoria", memory).is_err() {
                        break;
                    }

                    if app_handle.emit("evento-opencode", opencode_active).is_err() {
                        break;
                    }

                    if app_handle
                        .emit("evento-opencode-cpu", opencode_cpu)
                        .is_err()
                    {
                        break;
                    }
                }
            });

            tauri::async_runtime::spawn(run_telegram(app.handle().clone()));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running HAL");
}
