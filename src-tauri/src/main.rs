#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{thread, time::Duration};

use sysinfo::{ProcessesToUpdate, System};
use tauri::{Emitter, Manager, PhysicalPosition, Position};

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
                    system.refresh_processes(ProcessesToUpdate::All, true);
                    let cpu = system.global_cpu_usage();
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

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running HAL");
}
