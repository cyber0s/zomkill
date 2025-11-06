#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::sync::{Arc, Mutex};

#[cfg(target_os = "windows")]
use windows::{
    core::*, Win32::Foundation::*, Win32::System::Diagnostics::Debug::*,
    Win32::System::Threading::*, Win32::UI::WindowsAndMessaging::*,
};

const GAME_WINDOW_CLASS: &str = "MainWindow";
const GAME_WINDOW_TITLE: &str = "Plants vs. Zombies";

// 内存地址配置
const SUN_BASE_ADDR: u32 = 0x00755E0C;
const SUN_OFFSET_1: u32 = 0x868;
const SUN_OFFSET_2: u32 = 0x5578;

const COOLDOWN_BASE_ADDR: u32 = 0x00755E0C;
const COOLDOWN_OFFSET_1: u32 = 0x868;
const COOLDOWN_OFFSET_2: u32 = 0x15C;
const COOLDOWN_OFFSET_3: u32 = 0x70;

// 设置自定义字体
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 加载中文字体（编译到二进制中）
    fonts.font_data.insert(
        "noto_sans_sc".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSansSC-Regular.ttf")),
    );

    // 设置字体优先级
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "noto_sans_sc".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("noto_sans_sc".to_owned());

    ctx.set_fonts(fonts);
}

#[derive(Clone, Copy, PartialEq)]
enum GameStatus {
    NotRunning,
    Running,
    InLevel,
}

struct ZomKillApp {
    game_status: GameStatus,
    process_id: u32,
    current_sun: i32,
    new_sun_value: String,
    no_cooldown_active: Arc<Mutex<bool>>,
    status_message: String,
    show_about: bool,
}

impl Default for ZomKillApp {
    fn default() -> Self {
        Self {
            game_status: GameStatus::NotRunning,
            process_id: 0,
            current_sun: 0,
            new_sun_value: String::from("9990"),
            no_cooldown_active: Arc::new(Mutex::new(false)),
            status_message: String::from("等待游戏启动..."),
            show_about: false,
        }
    }
}

impl ZomKillApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 加载中文字体
        setup_custom_fonts(&cc.egui_ctx);
        Self::default()
    }

    #[cfg(target_os = "windows")]
    fn check_game_status(&mut self) {
        unsafe {
            let class_name = GAME_WINDOW_CLASS
                .encode_utf16()
                .chain(Some(0))
                .collect::<Vec<_>>();
            let window_title = GAME_WINDOW_TITLE
                .encode_utf16()
                .chain(Some(0))
                .collect::<Vec<_>>();

            let hwnd = FindWindowW(PCWSTR(class_name.as_ptr()), PCWSTR(window_title.as_ptr()));

            match hwnd {
                Ok(h) if !h.is_invalid() => {
                    let mut pid: u32 = 0;
                    GetWindowThreadProcessId(h, Some(&mut pid));
                    self.process_id = pid;
                    self.game_status = GameStatus::Running;
                    self.status_message = format!("游戏已运行 - 进程ID: {}", pid);
                }
                _ => {
                    self.game_status = GameStatus::NotRunning;
                    self.process_id = 0;
                    self.status_message = "游戏未运行，请启动《植物大战僵尸年度版》".to_string();
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn check_game_status(&mut self) {
        self.status_message = "此程序仅支持 Windows 系统".to_string();
    }

    #[cfg(target_os = "windows")]
    fn read_sun_value(&mut self) -> bool {
        unsafe {
            let h_process = OpenProcess(PROCESS_ALL_ACCESS, false, self.process_id);

            if let Ok(handle) = h_process {
                if handle.is_invalid() {
                    self.status_message = "无法打开游戏进程，请确保已进入关卡".to_string();
                    return false;
                }

                // 读取基址
                let mut base_value: u32 = 0;
                let mut bytes_read = 0;
                if ReadProcessMemory(
                    handle,
                    SUN_BASE_ADDR as *const _,
                    &mut base_value as *mut _ as *mut _,
                    4,
                    Some(&mut bytes_read),
                )
                .is_err()
                {
                    CloseHandle(handle).ok();
                    return false;
                }

                // 读取第一级偏移
                let mut offset1_value: u32 = 0;
                if ReadProcessMemory(
                    handle,
                    (base_value + SUN_OFFSET_1) as *const _,
                    &mut offset1_value as *mut _ as *mut _,
                    4,
                    Some(&mut bytes_read),
                )
                .is_err()
                {
                    CloseHandle(handle).ok();
                    return false;
                }

                // 读取最终阳光值
                let mut sun_value: i32 = 0;
                if ReadProcessMemory(
                    handle,
                    (offset1_value + SUN_OFFSET_2) as *const _,
                    &mut sun_value as *mut _ as *mut _,
                    4,
                    Some(&mut bytes_read),
                )
                .is_ok()
                {
                    self.current_sun = sun_value;
                    CloseHandle(handle).ok();
                    return true;
                }

                CloseHandle(handle).ok();
            }
        }
        false
    }

    #[cfg(not(target_os = "windows"))]
    fn read_sun_value(&mut self) -> bool {
        false
    }

    #[cfg(target_os = "windows")]
    fn write_sun_value(&mut self) -> bool {
        if let Ok(value) = self.new_sun_value.parse::<i32>() {
            unsafe {
                let h_process = OpenProcess(PROCESS_ALL_ACCESS, false, self.process_id);

                if let Ok(handle) = h_process {
                    if handle.is_invalid() {
                        self.status_message = "无法打开游戏进程".to_string();
                        return false;
                    }

                    let mut base_value: u32 = 0;
                    let mut bytes_read = 0;
                    ReadProcessMemory(
                        handle,
                        SUN_BASE_ADDR as *const _,
                        &mut base_value as *mut _ as *mut _,
                        4,
                        Some(&mut bytes_read),
                    )
                    .ok();

                    let mut offset1_value: u32 = 0;
                    ReadProcessMemory(
                        handle,
                        (base_value + SUN_OFFSET_1) as *const _,
                        &mut offset1_value as *mut _ as *mut _,
                        4,
                        Some(&mut bytes_read),
                    )
                    .ok();

                    let mut bytes_written = 0;
                    if WriteProcessMemory(
                        handle,
                        (offset1_value + SUN_OFFSET_2) as *const _,
                        &value as *const _ as *const _,
                        4,
                        Some(&mut bytes_written),
                    )
                    .is_ok()
                    {
                        self.current_sun = value;
                        self.status_message = format!("✓ 阳光值修改成功: {}", value);
                        CloseHandle(handle).ok();
                        return true;
                    }

                    CloseHandle(handle).ok();
                }
            }
        } else {
            self.status_message = "请输入有效的数字".to_string();
        }
        false
    }

    #[cfg(not(target_os = "windows"))]
    fn write_sun_value(&mut self) -> bool {
        false
    }

    #[cfg(target_os = "windows")]
    fn toggle_no_cooldown(&mut self) {
        let active = Arc::clone(&self.no_cooldown_active);
        let current_active = *active.lock().unwrap();

        if current_active {
            // 停止无冷却
            *active.lock().unwrap() = false;
            self.status_message = "✓ 已停止无冷却功能".to_string();
        } else {
            // 启动无冷却
            *active.lock().unwrap() = true;
            self.status_message = "✓ 无冷却功能已激活".to_string();

            let pid = self.process_id;
            std::thread::spawn(move || {
                unsafe {
                    while *active.lock().unwrap() {
                        if let Ok(handle) = OpenProcess(PROCESS_ALL_ACCESS, false, pid) {
                            if !handle.is_invalid() {
                                let mut base_value: u32 = 0;
                                let mut bytes_read = 0;

                                ReadProcessMemory(
                                    handle,
                                    COOLDOWN_BASE_ADDR as *const _,
                                    &mut base_value as *mut _ as *mut _,
                                    4,
                                    Some(&mut bytes_read),
                                )
                                .ok();

                                let mut offset1_value: u32 = 0;
                                ReadProcessMemory(
                                    handle,
                                    (base_value + COOLDOWN_OFFSET_1) as *const _,
                                    &mut offset1_value as *mut _ as *mut _,
                                    4,
                                    Some(&mut bytes_read),
                                )
                                .ok();

                                let mut offset2_value: u32 = 0;
                                ReadProcessMemory(
                                    handle,
                                    (offset1_value + COOLDOWN_OFFSET_2) as *const _,
                                    &mut offset2_value as *mut _ as *mut _,
                                    4,
                                    Some(&mut bytes_read),
                                )
                                .ok();

                                let value: i32 = 1;
                                let mut bytes_written = 0;

                                // 写入所有卡槽
                                for i in 0..10 {
                                    let addr = offset2_value + COOLDOWN_OFFSET_3 + (i * 0x50);
                                    WriteProcessMemory(
                                        handle,
                                        addr as *const _,
                                        &value as *const _ as *const _,
                                        4,
                                        Some(&mut bytes_written),
                                    )
                                    .ok();
                                }

                                CloseHandle(handle).ok();
                            }
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            });
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn toggle_no_cooldown(&mut self) {
        self.status_message = "此功能仅支持 Windows 系统".to_string();
    }
}

impl eframe::App for ZomKillApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 自动检测游戏状态
        self.check_game_status();

        // 如果游戏运行中，自动读取阳光值
        if self.game_status != GameStatus::NotRunning {
            self.read_sun_value();
        }

        // 顶部面板
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.heading("🌻 ZomKill v2.0");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("关于").clicked() {
                        self.show_about = !self.show_about;
                    }
                });
            });
            ui.add_space(5.0);
        });

        // 底部状态栏
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                let status_color = match self.game_status {
                    GameStatus::NotRunning => egui::Color32::RED,
                    GameStatus::Running => egui::Color32::GREEN,
                    GameStatus::InLevel => egui::Color32::BLUE,
                };

                ui.colored_label(status_color, "●");
                ui.label(&self.status_message);
            });
            ui.add_space(5.0);
        });

        // 中央面板
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);

            // 游戏状态显示
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    ui.heading("游戏状态");
                    ui.add_space(10.0);

                    if self.game_status == GameStatus::NotRunning {
                        ui.colored_label(egui::Color32::RED, "❌ 游戏未运行");
                        ui.label("请启动《植物大战僵尸年度版》");
                    } else {
                        ui.colored_label(egui::Color32::GREEN, "✓ 游戏已连接");
                        ui.label(format!("进程ID: {}", self.process_id));
                        ui.label(format!("当前阳光: {}", self.current_sun));
                    }
                });
            });

            ui.add_space(20.0);

            // 功能区域
            let enabled = self.game_status != GameStatus::NotRunning;

            // 阳光修改
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    ui.heading("🌞 阳光修改");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label("设置阳光值:");
                        ui.add_enabled(
                            enabled,
                            egui::TextEdit::singleline(&mut self.new_sun_value)
                                .desired_width(150.0),
                        );
                    });

                    ui.add_space(10.0);

                    if ui
                        .add_enabled(
                            enabled,
                            egui::Button::new("🎯 修改阳光").min_size(egui::vec2(150.0, 30.0)),
                        )
                        .clicked()
                    {
                        self.write_sun_value();
                    }
                });
            });

            ui.add_space(10.0);

            // 无冷却功能
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    ui.heading("⚡ 植物卡槽");
                    ui.add_space(10.0);

                    let no_cooldown_active = *self.no_cooldown_active.lock().unwrap();
                    let button_text = if no_cooldown_active {
                        "🛑 停止无冷却"
                    } else {
                        "🚀 启动无冷却"
                    };

                    let button_color = if no_cooldown_active {
                        egui::Color32::from_rgb(200, 50, 50)
                    } else {
                        egui::Color32::from_rgb(50, 150, 50)
                    };

                    if ui
                        .add_enabled(
                            enabled,
                            egui::Button::new(button_text)
                                .fill(button_color)
                                .min_size(egui::vec2(150.0, 30.0)),
                        )
                        .clicked()
                    {
                        self.toggle_no_cooldown();
                    }

                    ui.add_space(5.0);
                    ui.label("(持续为所有植物卡槽移除冷却)");
                });
            });

            ui.add_space(20.0);

            // 快捷按钮
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    ui.heading("⚡ 快捷操作");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                enabled,
                                egui::Button::new("9990☀").min_size(egui::vec2(80.0, 25.0)),
                            )
                            .clicked()
                        {
                            self.new_sun_value = "9990".to_string();
                            self.write_sun_value();
                        }
                        if ui
                            .add_enabled(
                                enabled,
                                egui::Button::new("8000☀").min_size(egui::vec2(80.0, 25.0)),
                            )
                            .clicked()
                        {
                            self.new_sun_value = "8000".to_string();
                            self.write_sun_value();
                        }
                        if ui
                            .add_enabled(
                                enabled,
                                egui::Button::new("5000☀").min_size(egui::vec2(80.0, 25.0)),
                            )
                            .clicked()
                        {
                            self.new_sun_value = "5000".to_string();
                            self.write_sun_value();
                        }
                    });
                });
            });
        });

        // 关于对话框
        if self.show_about {
            egui::Window::new("关于 ZomKill")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("🌻 ZomKill v3.0");
                        ui.add_space(10.0);
                        ui.label("植物大战僵尸辅助工具");
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);
                        ui.label("功能特性:");
                        ui.label("• 阳光值修改");
                        ui.label("• 植物卡槽无冷却");
                        ui.label("• 现代化图形界面");
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);
                        ui.label("使用 Rust + egui 重写");
                        ui.label("仅支持《植物大战僵尸年度版》");
                        ui.add_space(10.0);
                        ui.label("作者: Xinux");
                        ui.hyperlink_to("www.xinux.top", "https://www.xinux.top");
                        ui.add_space(10.0);
                        if ui.button("关闭").clicked() {
                            self.show_about = false;
                        }
                    });
                });
        }

        // 请求持续刷新
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([450.0, 650.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "ZomKill v2.0 - 植物大战僵尸辅助",
        options,
        Box::new(|cc| Ok(Box::new(ZomKillApp::new(cc)))),
    )
}
