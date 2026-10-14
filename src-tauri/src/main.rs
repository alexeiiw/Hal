#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env, fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use chrono::{Duration as ChronoDuration, Local, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sysinfo::{ProcessesToUpdate, System};
use tauri::{Emitter, Manager, PhysicalPosition, Position};

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct TelegramConfig {
    token: String,
    chat_id: Option<i64>,
    offset: Option<i64>,
    mode: String,
    task: Option<String>,
    disabled_views: Vec<String>,
}

#[derive(Clone, Default)]
struct SystemStatus {
    cpu: f32,
    memory: f32,
}

#[derive(Clone, Deserialize, Serialize)]
struct Reminder {
    label: String,
    due_at: i64,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct ReminderStore {
    reminders: Vec<Reminder>,
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

#[derive(Deserialize)]
struct BitcoinPriceResponse {
    bitcoin: Option<BitcoinPrice>,
}

#[derive(Deserialize)]
struct BitcoinPrice {
    usd: Option<f64>,
}

fn telegram_config_path() -> PathBuf {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("HAL")
        .join("telegram.json")
}

fn reminders_path() -> PathBuf {
    telegram_config_path()
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("reminders.json")
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

fn load_reminders(path: &PathBuf) -> ReminderStore {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

fn save_reminders(path: &PathBuf, store: &ReminderStore) {
    if let Ok(content) = serde_json::to_string_pretty(store) {
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

async fn bitcoin_price(client: &reqwest::Client) -> Option<f64> {
    let response = client
        .get("https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd")
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;
    response
        .json::<BitcoinPriceResponse>()
        .await
        .ok()?
        .bitcoin?
        .usd
}

fn inventory_status(config: &TelegramConfig) -> String {
    let views = ["cpu", "ram", "clima", "reloj", "btc", "cara", "tarea"];
    let active = views
        .iter()
        .filter(|view| {
            let name = **view;
            let internal = match name {
                "ram" => "memory",
                "clima" => "weather",
                "reloj" => "clock",
                "btc" => "bitcoin",
                "cara" => "face",
                "tarea" => "task",
                view => view,
            };
            !config
                .disabled_views
                .iter()
                .any(|disabled| disabled == internal)
        })
        .copied()
        .collect::<Vec<_>>()
        .join(", ");
    let disabled = if config.disabled_views.is_empty() {
        "ninguna".to_string()
    } else {
        config.disabled_views.join(", ")
    };
    format!("Activas: {active}\nDesactivadas: {disabled}")
}

fn parse_reminder(value: &str) -> Result<Reminder, String> {
    if let Ok(minutes) = value.parse::<i64>() {
        if !(1..=1_440).contains(&minutes) {
            return Err("El aviso debe estar entre 1 y 1440 minutos.".to_string());
        }
        return Ok(Reminder {
            label: format!("{minutes} MIN"),
            due_at: (Utc::now() + ChronoDuration::minutes(minutes)).timestamp(),
        });
    }

    let Some((hour, minute)) = value.split_once(':') else {
        return Err("Usa /aviso 10 o /aviso 08:00.".to_string());
    };
    let (Ok(hour), Ok(minute)) = (hour.parse::<u32>(), minute.parse::<u32>()) else {
        return Err("La hora debe tener formato HH:MM.".to_string());
    };
    if hour > 23 || minute > 59 {
        return Err("La hora debe estar entre 00:00 y 23:59.".to_string());
    }

    let now = Local::now();
    let mut due = Local
        .from_local_datetime(
            &now.date_naive()
                .and_hms_opt(hour, minute, 0)
                .ok_or_else(|| "La hora no es válida.".to_string())?,
        )
        .single()
        .ok_or_else(|| "No se pudo interpretar la hora local.".to_string())?;
    if due <= now {
        due = Local
            .from_local_datetime(
                &(now.date_naive() + ChronoDuration::days(1))
                    .and_hms_opt(hour, minute, 0)
                    .ok_or_else(|| "La hora no es válida.".to_string())?,
            )
            .single()
            .ok_or_else(|| "No se pudo interpretar la hora local.".to_string())?;
    }

    Ok(Reminder {
        label: format!("{hour:02}:{minute:02}"),
        due_at: due.timestamp(),
    })
}

async fn handle_telegram_command(
    app: &tauri::AppHandle,
    client: &reqwest::Client,
    config: &mut TelegramConfig,
    system_status: &Arc<Mutex<SystemStatus>>,
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
        "Comandos:\n/pair\n/inventory\n/mode normal|developer|focus|relax|happy\n/on nombre\n/off nombre\n/off hal\n/phrase texto\n/task texto\n/aviso 10 o 08:00\n/aviso list|cancel\n/status"
            .to_string()
    } else if text == "/inventory" {
        format!(
            "{}\nModos: normal, developer, focus, relax, happy\nUsa /on nombre o /off nombre.",
            inventory_status(config)
        )
    } else if text == "/status" {
        let status = system_status
            .lock()
            .map(|status| status.clone())
            .unwrap_or_default();
        let bitcoin = bitcoin_price(client)
            .await
            .map(|price| format!("BTC ${price:.0}"))
            .unwrap_or_else(|| "BTC NO DISP.".to_string());
        let task = config.task.as_deref().unwrap_or("sin tarea");
        format!(
            "CPU {:.1}% | RAM {:.1}%\n{}\nModo: {}\nTarea: {}\n{}",
            status.cpu,
            status.memory,
            bitcoin,
            if config.mode.is_empty() {
                "normal"
            } else {
                &config.mode
            },
            task,
            inventory_status(config)
        )
    } else if let Some(mode) = text.strip_prefix("/mode ") {
        match mode.trim() {
            "normal" | "developer" | "focus" | "relax" | "happy" => {
                config.mode = mode.trim().to_string();
                let _ = app.emit("evento-modo", mode.trim());
                format!("Modo {} activado.", mode.trim())
            }
            _ => "Modo inválido. Usa normal, developer, focus, relax o happy.".to_string(),
        }
    } else if let Some(view) = text.strip_prefix("/on ") {
        set_view(app, config, view.trim(), true)
    } else if let Some(view) = text.strip_prefix("/off ") {
        set_view(app, config, view.trim(), false)
    } else if text == "/task" {
        "Usa /task seguido de una tarea de hasta 30 caracteres.".to_string()
    } else if let Some(task) = text.strip_prefix("/task ") {
        let task = task.trim();

        if task == "clear" {
            config.task = None;
            let _ = app.emit("evento-tarea", Option::<String>::None);
            "Tarea eliminada.".to_string()
        } else if task.is_empty() || task.chars().count() > 30 {
            "La tarea debe tener entre 1 y 30 caracteres.".to_string()
        } else {
            config.task = Some(task.to_string());
            let _ = app.emit("evento-tarea", Some(task.to_string()));
            "Tarea prioritaria actualizada.".to_string()
        }
    } else if text == "/aviso" {
        "Usa /aviso 10 para minutos o /aviso 08:00 para una hora local.".to_string()
    } else if text == "/aviso list" {
        let store = load_reminders(&reminders_path());
        if store.reminders.is_empty() {
            "No hay avisos activos.".to_string()
        } else {
            let labels = store
                .reminders
                .iter()
                .map(|reminder| reminder.label.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("Avisos activos: {labels}")
        }
    } else if text == "/aviso cancel" {
        save_reminders(&reminders_path(), &ReminderStore::default());
        "Avisos cancelados.".to_string()
    } else if let Some(value) = text.strip_prefix("/aviso ") {
        match parse_reminder(value.trim()) {
            Ok(reminder) => {
                let path = reminders_path();
                let mut store = load_reminders(&path);
                let label = reminder.label.clone();
                store.reminders.push(reminder);
                save_reminders(&path, &store);
                format!("Aviso programado para {label}.")
            }
            Err(message) => message,
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

fn set_view(
    app: &tauri::AppHandle,
    config: &mut TelegramConfig,
    view: &str,
    enabled: bool,
) -> String {
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
            if enabled {
                config.disabled_views.retain(|disabled| disabled != view);
            } else if !config
                .disabled_views
                .iter()
                .any(|disabled| disabled == view)
            {
                config.disabled_views.push(view.to_string());
            }
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

async fn run_telegram(app: tauri::AppHandle, system_status: Arc<Mutex<SystemStatus>>) {
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
                handle_telegram_command(&app, &client, &mut config, &system_status, message).await;
            }
        }
        save_telegram_config(&path, &config);
    }
}

fn run_reminder_watcher(app: tauri::AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(5));
        let path = reminders_path();
        let mut store = load_reminders(&path);
        let now = Utc::now().timestamp();
        let due = store
            .reminders
            .iter()
            .filter(|reminder| reminder.due_at <= now)
            .cloned()
            .collect::<Vec<_>>();

        if due.is_empty() {
            continue;
        }

        store.reminders.retain(|reminder| reminder.due_at > now);
        save_reminders(&path, &store);

        for reminder in due {
            let _ = app.emit("evento-aviso", reminder.label.clone());
            let config = load_telegram_config(&telegram_config_path());
            if let Some(config) = config {
                if let Some(chat_id) = config.chat_id {
                    if !config.token.is_empty() {
                        let token = config.token;
                        let label = reminder.label;
                        tauri::async_runtime::spawn(async move {
                            telegram_send(
                                &reqwest::Client::new(),
                                &token,
                                chat_id,
                                &format!("Aviso cumplido: {label}"),
                            )
                            .await;
                        });
                    }
                }
            }
        }
    });
}

async fn run_bitcoin_alert(app: tauri::AppHandle) {
    let client = reqwest::Client::new();
    let mut was_below_threshold = false;

    loop {
        if let Some(price) = bitcoin_price(&client).await {
            let below_threshold = price <= 76_000.0;
            if below_threshold && !was_below_threshold {
                let config = load_telegram_config(&telegram_config_path());
                if let Some(config) = config {
                    if let Some(chat_id) = config.chat_id {
                        if !config.token.is_empty() {
                            telegram_send(
                                &client,
                                &config.token,
                                chat_id,
                                &format!("ALERTA BTC: ${price:.0} esta en o bajo $76000."),
                            )
                            .await;
                        }
                    }
                }
                let _ = app.emit("evento-btc-alerta", true);
            }
            was_below_threshold = below_threshold;
        }

        tokio::time::sleep(Duration::from_secs(6 * 60 * 60)).await;
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

            let system_status = Arc::new(Mutex::new(SystemStatus::default()));
            let monitor_status = system_status.clone();
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
                    if let Ok(mut status) = monitor_status.lock() {
                        status.cpu = cpu;
                        status.memory = memory;
                    }
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

            tauri::async_runtime::spawn(run_telegram(app.handle().clone(), system_status));
            run_reminder_watcher(app.handle().clone());
            tauri::async_runtime::spawn(run_bitcoin_alert(app.handle().clone()));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running HAL");
}
