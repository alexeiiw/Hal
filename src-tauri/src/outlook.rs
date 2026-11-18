use std::{
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use chrono::{Local, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::{load_telegram_config, telegram_config_path, telegram_send};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const BRIDGE_SCRIPT: &str = include_str!("../resources/outlook-bridge.ps1");
const DEFAULT_ALERT_MINUTES: u32 = 15;
const MAIL_ALERT_COOLDOWN: Duration = Duration::from_secs(120);
const STALE_BRIDGE: Duration = Duration::from_secs(240);

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgendaEvent {
    pub id: String,
    pub subject: String,
    pub start: i64,
    pub end: i64,
    pub all_day: bool,
}

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlookStatus {
    pub running: bool,
    pub ok: bool,
    pub error: Option<String>,
    pub unread: u32,
    pub today: u32,
    pub tomorrow: u32,
    pub next: Option<AgendaEvent>,
    pub agenda: Vec<AgendaEvent>,
    pub fetched_at: i64,
}

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MailPreview {
    pub sender: String,
    pub subject: String,
    pub unread: bool,
}

#[derive(Clone, Serialize)]
struct OutlookAviso {
    kind: &'static str,
    text: String,
}

#[derive(Default, Deserialize)]
struct BridgeLine {
    #[serde(default)]
    running: bool,
    #[serde(default)]
    ok: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    unread: u32,
    #[serde(default)]
    events: Vec<BridgeEvent>,
    #[serde(default)]
    mail: Vec<BridgeMail>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BridgeEvent {
    #[serde(default)]
    id: String,
    #[serde(default)]
    subject: String,
    #[serde(default)]
    start: i64,
    #[serde(default)]
    end: i64,
    #[serde(default)]
    all_day: bool,
}

#[derive(Default, Deserialize)]
struct BridgeMail {
    #[serde(default)]
    id: String,
    #[serde(default)]
    sender: String,
    #[serde(default)]
    subject: String,
    #[serde(default)]
    received: i64,
    #[serde(default)]
    unread: bool,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct OutlookState {
    alerted: Vec<String>,
    seen_mail: Vec<String>,
}

fn state_path() -> PathBuf {
    telegram_config_path().with_file_name("outlook-avisos.json")
}

fn script_path() -> PathBuf {
    telegram_config_path().with_file_name("outlook-bridge.ps1")
}

fn load_state() -> OutlookState {
    fs::read_to_string(state_path())
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

fn save_state(state: &OutlookState) {
    if let Ok(content) = serde_json::to_string_pretty(state) {
        let _ = fs::write(state_path(), content);
    }
}

fn ensure_script() -> Option<PathBuf> {
    let path = script_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok()?;
    }
    fs::write(&path, BRIDGE_SCRIPT).ok()?;
    Some(path)
}

pub fn clip(text: &str, max: usize) -> String {
    let trimmed = text.trim();
    let mut chars = trimmed.chars();
    let collected = chars.by_ref().take(max).collect::<String>();
    if chars.next().is_some() {
        format!("{collected}…")
    } else {
        collected
    }
}

pub fn clock(start: i64) -> String {
    Local
        .timestamp_opt(start, 0)
        .single()
        .map(|value| value.format("%H:%M").to_string())
        .unwrap_or_else(|| "--:--".to_string())
}

fn day_key(timestamp: i64) -> String {
    Local
        .timestamp_opt(timestamp, 0)
        .single()
        .map(|value| value.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

fn build_status(line: &BridgeLine) -> OutlookStatus {
    let now = Utc::now().timestamp();
    let now_local = Local::now();
    let today = now_local.format("%Y-%m-%d").to_string();
    let tomorrow = (now_local + chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    let mut events = line
        .events
        .iter()
        .map(|event| AgendaEvent {
            id: event.id.clone(),
            subject: event.subject.clone(),
            start: event.start,
            end: event.end,
            all_day: event.all_day,
        })
        .filter(|event| event.end > now - 60)
        .collect::<Vec<_>>();
    events.sort_by_key(|event| event.start);

    let next = events
        .iter()
        .find(|event| !event.all_day && event.end > now)
        .or_else(|| events.first())
        .cloned();

    let today_count = events
        .iter()
        .filter(|event| !event.all_day && day_key(event.start) == today && event.end > now)
        .count() as u32;

    let tomorrow_count = events
        .iter()
        .filter(|event| !event.all_day && day_key(event.start) == tomorrow && event.end > now)
        .count() as u32;

    let agenda = events
        .iter()
        .filter(|event| event.end > now)
        .take(3)
        .cloned()
        .collect::<Vec<_>>();

    OutlookStatus {
        running: line.running,
        ok: line.ok,
        error: line.error.clone(),
        unread: line.unread,
        today: today_count,
        tomorrow: tomorrow_count,
        next,
        agenda,
        fetched_at: now,
    }
}

fn config_value() -> Option<crate::TelegramConfig> {
    load_telegram_config(&telegram_config_path())
}

pub fn alert_minutes() -> u32 {
    config_value()
        .and_then(|config| config.outlook_alert_minutes)
        .unwrap_or(DEFAULT_ALERT_MINUTES)
        .min(60)
}

fn mail_preview_mode() -> String {
    config_value()
        .and_then(|config| config.mail_preview)
        .unwrap_or_else(|| "ambos".to_string())
}

fn mail_alert_enabled() -> bool {
    config_value()
        .and_then(|config| config.mail_alert)
        .unwrap_or(false)
}

fn telegram_push(text: String) {
    let Some(config) = config_value() else { return };
    if config.token.trim().is_empty() {
        return;
    }
    let Some(chat_id) = config.chat_id else { return };
    let token = config.token;
    tauri::async_runtime::spawn(async move {
        let Ok(client) = reqwest::Client::builder().user_agent("HAL/1.9.1").build() else {
            return;
        };
        telegram_send(&client, &token, chat_id, &text).await;
    });
}

fn handle_alerts(line: &BridgeLine, state: &mut OutlookState, last_mail: &mut Option<Instant>, app: &AppHandle) {
    let now = Utc::now().timestamp();
    let mut dirty = false;

    let minutes = alert_minutes();
    if minutes > 0 {
        for event in line.events.iter().filter(|event| !event.all_day) {
            let remaining = event.start - now;
            if remaining <= 0 || remaining > (minutes as i64 * 60) {
                continue;
            }

            let key = format!("{}@{}", event.id, event.start);
            if state.alerted.iter().any(|value| value == &key) {
                continue;
            }

            state.alerted.push(key);
            dirty = true;

            let text = format!("{} {}", clock(event.start), clip(&event.subject.to_uppercase(), 20));
            let _ = app.emit(
                "evento-outlook-aviso",
                OutlookAviso {
                    kind: "cita",
                    text: text.clone(),
                },
            );
            telegram_push(format!("{text} · en {minutes} min"));
        }
    }

    let first_poll = state.seen_mail.is_empty();

    let newest = line
        .mail
        .iter()
        .filter(|mail| mail.unread && now - mail.received < 600)
        .filter(|mail| !state.seen_mail.iter().any(|value| value == &mail.id))
        .collect::<Vec<_>>();

    for mail in &line.mail {
        if !mail.id.is_empty() && !state.seen_mail.iter().any(|value| value == &mail.id) {
            state.seen_mail.push(mail.id.clone());
            dirty = true;
        }
    }
    if state.seen_mail.len() > 60 {
        let excess = state.seen_mail.len() - 60;
        state.seen_mail.drain(0..excess);
    }

    let preview = mail_preview_mode();
    if !first_poll && !newest.is_empty() && preview != "off" {
        let cooled = last_mail
            .map(|instant| instant.elapsed() >= MAIL_ALERT_COOLDOWN)
            .unwrap_or(true);

        if cooled {
            *last_mail = Some(Instant::now());
            let text = if newest.len() > 1 {
                format!("{} NUEVOS · {}", newest.len(), clip(&newest[0].sender.to_uppercase(), 8))
            } else {
                let mail = newest[0];
                match preview.as_str() {
                    "remitente" => clip(&mail.sender.to_uppercase(), 18),
                    "asunto" => clip(&mail.subject.to_uppercase(), 18),
                    _ => format!(
                        "{}|{}",
                        clip(&mail.sender.to_uppercase(), 16),
                        clip(&mail.subject.to_uppercase(), 16)
                    ),
                }
            };
            let _ = app.emit(
                "evento-outlook-aviso",
                OutlookAviso {
                    kind: "mail",
                    text: text.clone(),
                },
            );
            if mail_alert_enabled() {
                telegram_push(format!("CORREO · {}", text.replace('|', " · ")));
            }
        }
    }

    if dirty {
        prune_state(state, now);
        save_state(state);
    }
}

fn prune_state(state: &mut OutlookState, now: i64) {
    let limit = now - 2 * 24 * 60 * 60;
    state.alerted.retain(|key| {
        key.rsplit('@')
            .next()
            .and_then(|value| value.parse::<i64>().ok())
            .map(|start| start >= limit)
            .unwrap_or(false)
    });
}

fn spawn_bridge(script: &PathBuf, parent: u32) -> Option<Child> {
    let mut command = Command::new("powershell.exe");
    command
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-File",
        ])
        .arg(script)
        .arg("-ParentPid")
        .arg(parent.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    command.spawn().ok()
}

fn kill_tree(pid: u32) {
    let mut command = Command::new("taskkill");
    command.args(["/PID", &pid.to_string(), "/F"]);
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    let _ = command.output();
}

fn run_once(
    app: &AppHandle,
    status: &Arc<Mutex<OutlookStatus>>,
    mail_cache: &Arc<Mutex<Vec<MailPreview>>>,
    state: &mut OutlookState,
    last_mail: &mut Option<Instant>,
    script: &PathBuf,
) {
    let Some(mut child) = spawn_bridge(script, std::process::id()) else {
        thread::sleep(Duration::from_secs(30));
        return;
    };

    let Some(stdout) = child.stdout.take() else {
        let _ = child.kill();
        return;
    };

    let alive = Arc::new(AtomicBool::new(true));
    let last_line = Arc::new(Mutex::new(Instant::now()));
    let pid = child.id();

    {
        let alive = alive.clone();
        let last_line = last_line.clone();
        thread::spawn(move || {
            while alive.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_secs(30));
                if !alive.load(Ordering::Relaxed) {
                    break;
                }
                let stale = last_line
                    .lock()
                    .map(|instant| instant.elapsed() > STALE_BRIDGE)
                    .unwrap_or(false);
                if stale {
                    kill_tree(pid);
                    break;
                }
            }
        });
    }

    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
        if let Ok(mut instant) = last_line.lock() {
            *instant = Instant::now();
        }

        let trimmed = line.trim_start_matches('\u{feff}').trim();
        if trimmed.is_empty() || !trimmed.starts_with('{') {
            continue;
        }

        let Ok(parsed) = serde_json::from_str::<BridgeLine>(trimmed) else {
            continue;
        };

        let snapshot = build_status(&parsed);
        if let Ok(mut guard) = status.lock() {
            *guard = snapshot.clone();
        }
        let _ = app.emit("evento-outlook", snapshot);

        let mail = parsed
            .mail
            .iter()
            .map(|item| MailPreview {
                sender: item.sender.clone(),
                subject: item.subject.clone(),
                unread: item.unread,
            })
            .collect::<Vec<_>>();
        if let Ok(mut guard) = mail_cache.lock() {
            *guard = mail;
        }

        handle_alerts(&parsed, state, last_mail, app);
    }

    alive.store(false, Ordering::Relaxed);
    let _ = child.kill();
    let _ = child.wait();
    thread::sleep(Duration::from_secs(10));
}

fn agenda_enabled() -> bool {
    config_value()
        .map(|config| !config.disabled_views.iter().any(|view| view == "agenda"))
        .unwrap_or(true)
}

pub fn start(
    app: AppHandle,
    status: Arc<Mutex<OutlookStatus>>,
    mail_cache: Arc<Mutex<Vec<MailPreview>>>,
) {
    thread::spawn(move || {
        let Some(script) = ensure_script() else {
            return;
        };

        let mut state = load_state();
        let mut last_mail: Option<Instant> = None;

        loop {
            if !agenda_enabled() {
                thread::sleep(Duration::from_secs(60));
                continue;
            }
            run_once(&app, &status, &mail_cache, &mut state, &mut last_mail, &script);
        }
    });
}

pub fn snapshot(status: &Arc<Mutex<OutlookStatus>>) -> OutlookStatus {
    status.lock().map(|value| value.clone()).unwrap_or_default()
}

pub fn mail_snapshot(cache: &Arc<Mutex<Vec<MailPreview>>>) -> Vec<MailPreview> {
    cache.lock().map(|value| value.clone()).unwrap_or_default()
}

pub fn describe(status: &OutlookStatus) -> String {
    if !status.running {
        return "Outlook cerrado. HAL no lee datos.".to_string();
    }
    if !status.ok {
        return format!(
            "Outlook abierto, sin acceso: {}",
            status
                .error
                .clone()
                .unwrap_or_else(|| "error desconocido".to_string())
        );
    }

    let mut lines = vec![format!("Correo sin leer: {}", status.unread)];
    match &status.next {
        Some(next) => lines.push(format!(
            "Proxima: {} {}",
            clock(next.start),
            clip(&next.subject, 30)
        )),
        None => lines.push("Sin citas proximas.".to_string()),
    }
    lines.join("\n")
}

pub fn listing(status: &OutlookStatus) -> String {
    if !status.ok {
        return describe(status);
    }
    if status.agenda.is_empty() {
        return "Sin citas en las proximas 48 horas.".to_string();
    }

    let today = Local::now().format("%Y-%m-%d").to_string();
    status
        .agenda
        .iter()
        .map(|event| {
            let day = if day_key(event.start) == today {
                "hoy".to_string()
            } else {
                Local
                    .timestamp_opt(event.start, 0)
                    .single()
                    .map(|value| value.format("%d/%m").to_string())
                    .unwrap_or_default()
            };
            format!("{} {} · {}", day, clock(event.start), clip(&event.subject, 40))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn mail_listing(status: &OutlookStatus, mail: &[MailPreview]) -> String {
    let header = format!("Sin leer: {}", status.unread);
    if mail.is_empty() {
        return format!("{header}\nSin correo reciente.");
    }
    let list = mail
        .iter()
        .take(3)
        .map(|item| {
            format!(
                "{} {} · {}",
                if item.unread { "•" } else { " " },
                clip(&item.sender, 18),
                clip(&item.subject, 34)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("{header}\n{list}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(start: i64, subject: &str) -> BridgeEvent {
        BridgeEvent {
            id: format!("id{start}"),
            subject: subject.to_string(),
            start,
            end: start + 1800,
            all_day: false,
        }
    }

    #[test]
    fn clip_adds_ellipsis_only_when_needed() {
        assert_eq!(clip("HOLA", 8), "HOLA");
        assert_eq!(clip("REVISION PRESUPUESTO", 8), "REVISION…");
    }

    #[test]
    fn build_status_orders_and_counts() {
        let now = Utc::now().timestamp();
        let line = BridgeLine {
            running: true,
            ok: true,
            unread: 4,
            events: vec![event(now + 3600, "SEGUNDA"), event(now + 1200, "PRIMERA")],
            ..Default::default()
        };
        let status = build_status(&line);
        assert_eq!(status.unread, 4);
        assert_eq!(status.next.unwrap().subject, "PRIMERA");
        assert_eq!(status.agenda.len(), 2);
    }

    #[test]
    fn build_status_counts_tomorrow() {
        let now = Utc::now().timestamp();
        let line = BridgeLine {
            running: true,
            ok: true,
            events: vec![event(now + 86400, "MAÑANA")],
            ..Default::default()
        };
        let status = build_status(&line);
        assert_eq!(status.today, 0);
        assert_eq!(status.tomorrow, 1);
        assert_eq!(status.next.unwrap().subject, "MAÑANA");
    }

    #[test]
    fn build_status_ignores_finished_events() {
        let now = Utc::now().timestamp();
        let line = BridgeLine {
            running: true,
            ok: true,
            events: vec![event(now - 7200, "VIEJA")],
            ..Default::default()
        };
        let status = build_status(&line);
        assert!(status.next.is_none());
        assert!(status.agenda.is_empty());
    }

    #[test]
    fn build_status_skips_all_day_for_next() {
        let now = Utc::now().timestamp();
        let mut all_day = event(now - 3600, "TODO EL DIA");
        all_day.all_day = true;
        all_day.end = now + 3600;
        let line = BridgeLine {
            running: true,
            ok: true,
            events: vec![all_day, event(now + 1800, "REUNION")],
            ..Default::default()
        };
        let status = build_status(&line);
        assert_eq!(status.next.unwrap().subject, "REUNION");
    }

    #[test]
    fn prune_state_keeps_recent_keys() {
        let mut state = OutlookState {
            alerted: vec![
                "a@100".to_string(),
                format!("b@{}", Utc::now().timestamp()),
            ],
            seen_mail: Vec::new(),
        };
        prune_state(&mut state, Utc::now().timestamp());
        assert_eq!(state.alerted.len(), 1);
    }
}
