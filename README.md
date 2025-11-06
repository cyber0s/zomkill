# 🌻 使用 Rust 重写植物大战僵尸修改器 - ZomKill v2.0

> **作者**: Xinux

> **博客**: [www.xinux.top](https://www.xinux.top)

> **QQ**: 913499532

> **日期**: 2025-11-06,21:28

> **技术栈**: Rust + egui + Windows API

---

## 📝 前言

大家好，我是 Xinux！之前我用 C++ 写过一个植物大战僵尸的修改器（[查看原版文章](https://www.cnblogs.com/xinux/p/16273013.html)），功能包括修改阳光和移除植物卡槽冷却。最近学习了 Rust 语言，决定用 Rust 完全重写这个工具，并添加了现代化的图形界面。

**⚠️ 免责声明：本文代码仅供 Rust 编程学习研究，请勿用于非法用途！**

---

## ✨ 效果展示

### 功能特性

- 🌞 **阳光修改** - 自由修改游戏中的阳光数值
- ⚡ **无冷却功能** - 移除所有植物卡槽的冷却时间
- 🎨 **现代化界面** - 基于 egui 的美观图形化界面
- 🔄 **实时监控** - 自动检测游戏运行状态和当前阳光值
- ⚡ **快捷操作** - 一键设置常用阳光数值（9990/8000/5000）
- 🈯 **完美中文支持** - 内置思源黑体，中文显示完美

### 界面预览

程序界面简洁美观，包含：
- 顶部：程序标题和菜单栏
- 游戏状态区：显示游戏连接状态、进程ID、当前阳光值
- 阳光修改区：输入框和修改按钮
- 无冷却区：启动/停止无冷却功能
- 快捷操作区：一键设置常用阳光值
- 底部状态栏：实时显示操作状态和提示信息

![zomkill1.png](https://free.picui.cn/free/2025/11/06/690caa6cca503.png)
![zomkillabout.png](https://free.picui.cn/free/2025/11/06/690caa6d91669.png)




---

## 🎯 系统要求

- **操作系统**: Windows 7/8/10/11（64位）
- **游戏版本**: 植物大战僵尸年度版（Plants vs. Zombies Game of the Year Edition）
- **运行时**: 无需额外依赖，开箱即用

---

## 📥 下载使用

### 可执行文件下载

**游戏本体下载地址**: [植物大战僵尸游戏年度版文件](https://wwux.lanzouu.com/ihDIhrtonfi)
**辅助文件下载**: [zomkill v2.0辅助 约 29MB（包含内置中文字体）](https://wwux.lanzouu.com/iCwus3aawtni)

### 使用方法

1. 启动《植物大战僵尸年度版》
2. 双击运行 `zomkill.exe`
3. 程序会自动检测游戏并连接
4. 进入游戏关卡后即可使用功能

#### 阳光修改操作
- 在输入框中输入想要的阳光值（如 9990）
- 点击"🎯 修改阳光"按钮
- 或使用快捷按钮一键设置（9990☀ / 8000☀ / 5000☀）

#### 无冷却功能操作
- 点击"🚀 启动无冷却"按钮激活功能
- 所有植物卡槽将持续保持无冷却状态
- 点击"🛑 停止无冷却"可以关闭功能

---

## 💻 完整源代码

### 项目结构

```
zomkill/
├── Cargo.toml                      # 项目配置文件
├── build.bat                       # Windows 编译脚本
├── icon.ico                        # 程序图标
├── assets/
│   └── fonts/
│       └── NotoSansSC-Regular.ttf  # 中文字体
└── src/
    └── main.rs                     # 主程序源码
```

### Cargo.toml

```toml
[package]
name = "zomkill"
version = "2.0.0"
edition = "2021"

[dependencies]
eframe = "0.29"
egui = "0.29"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_System_Threading",
    "Win32_System_Diagnostics_Debug",
    "Win32_UI_WindowsAndMessaging",
] }
```

### 核心代码解析

#### 1. 内存地址定义

```rust
// 阳光值内存地址
const SUN_BASE_ADDR: u32 = 0x00755E0C;
const SUN_OFFSET_1: u32 = 0x868;
const SUN_OFFSET_2: u32 = 0x5578;

// 冷却时间内存地址
const COOLDOWN_BASE_ADDR: u32 = 0x00755E0C;
const COOLDOWN_OFFSET_1: u32 = 0x868;
const COOLDOWN_OFFSET_2: u32 = 0x15C;
const COOLDOWN_OFFSET_3: u32 = 0x70;
```

这些地址是通过 CE（Cheat Engine）工具分析游戏内存得到的。内存结构采用多级指针：
- **基址** + **偏移1** → 得到中间地址
- **中间地址** + **偏移2** → 得到最终数据地址

#### 2. 游戏进程检测

```rust
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

        let hwnd = FindWindowW(
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr())
        );

        match hwnd {
            Ok(h) if !h.is_invalid() => {
                let mut pid: u32 = 0;
                GetWindowThreadProcessId(h, Some(&mut pid));
                self.process_id = pid;
                self.game_status = GameStatus::Running;
            }
            _ => {
                self.game_status = GameStatus::NotRunning;
            }
        }
    }
}
```

**工作原理**：
1. 使用 `FindWindowW` 查找游戏窗口（通过窗口类名和标题）
2. 获取窗口句柄后，用 `GetWindowThreadProcessId` 获取进程ID
3. 更新游戏状态，供后续操作使用

#### 3. 读取阳光值

```rust
fn read_sun_value(&mut self) -> bool {
    unsafe {
        // 打开游戏进程
        let h_process = OpenProcess(PROCESS_ALL_ACCESS, false, self.process_id);

        if let Ok(handle) = h_process {
            // 读取基址
            let mut base_value: u32 = 0;
            let mut bytes_read = 0;
            ReadProcessMemory(
                handle,
                SUN_BASE_ADDR as *const _,
                &mut base_value as *mut _ as *mut _,
                4,
                Some(&mut bytes_read),
            )?;

            // 读取第一级偏移地址
            let mut offset1_value: u32 = 0;
            ReadProcessMemory(
                handle,
                (base_value + SUN_OFFSET_1) as *const _,
                &mut offset1_value as *mut _ as *mut _,
                4,
                Some(&mut bytes_read),
            )?;

            // 读取最终阳光值
            let mut sun_value: i32 = 0;
            ReadProcessMemory(
                handle,
                (offset1_value + SUN_OFFSET_2) as *const _,
                &mut sun_value as *mut _ as *mut _,
                4,
                Some(&mut bytes_read),
            )?;

            self.current_sun = sun_value;
            CloseHandle(handle)?;
            return true;
        }
    }
    false
}
```

**内存读取流程**：
```
基址 0x00755E0C
    ↓ 读取4字节
中间地址1 + 0x868
    ↓ 读取4字节
中间地址2 + 0x5578
    ↓ 读取4字节
最终阳光值 (int32)
```

#### 4. 修改阳光值

```rust
fn write_sun_value(&mut self) -> bool {
    if let Ok(value) = self.new_sun_value.parse::<i32>() {
        unsafe {
            let h_process = OpenProcess(PROCESS_ALL_ACCESS, false, self.process_id)?;

            // 读取多级指针（同上）
            let mut base_value: u32 = 0;
            ReadProcessMemory(/*...*/)?;

            let mut offset1_value: u32 = 0;
            ReadProcessMemory(/*...*/)?;

            // 写入新的阳光值
            let mut bytes_written = 0;
            WriteProcessMemory(
                handle,
                (offset1_value + SUN_OFFSET_2) as *const _,
                &value as *const _ as *const _,
                4,
                Some(&mut bytes_written),
            )?;

            self.current_sun = value;
            self.status_message = format!("✓ 阳光值修改成功: {}", value);
            CloseHandle(handle)?;
            return true;
        }
    }
    false
}
```

#### 5. 无冷却功能实现

```rust
fn toggle_no_cooldown(&mut self) {
    let active = Arc::clone(&self.no_cooldown_active);
    let current_active = *active.lock().unwrap();

    if !current_active {
        *active.lock().unwrap() = true;

        let pid = self.process_id;
        // 创建后台线程持续写入
        std::thread::spawn(move || {
            unsafe {
                while *active.lock().unwrap() {
                    if let Ok(handle) = OpenProcess(PROCESS_ALL_ACCESS, false, pid) {
                        // 读取多级指针
                        let mut base_value: u32 = 0;
                        ReadProcessMemory(/*...*/)?;

                        let mut offset1_value: u32 = 0;
                        ReadProcessMemory(/*...*/)?;

                        let mut offset2_value: u32 = 0;
                        ReadProcessMemory(/*...*/)?;

                        // 写入所有10个卡槽
                        let value: i32 = 1;
                        for i in 0..10 {
                            let addr = offset2_value + COOLDOWN_OFFSET_3 + (i * 0x50);
                            WriteProcessMemory(
                                handle,
                                addr as *const _,
                                &value as *const _ as *const _,
                                4,
                                Some(&mut bytes_written),
                            )?;
                        }

                        CloseHandle(handle)?;
                    }
                    // 每100毫秒刷新一次
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        });
    } else {
        // 停止无冷却
        *active.lock().unwrap() = false;
    }
}
```

**技术要点**：
- 使用 `Arc<Mutex<bool>>` 实现线程间的状态共享
- 后台线程每100ms写入一次，保持持续无冷却
- 每个卡槽的内存地址间隔 `0x50` 字节
- 写入值为 `1` 表示冷却完成状态

#### 6. 图形界面实现

```rust
impl eframe::App for ZomKillApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 自动检测游戏
        self.check_game_status();

        // 自动读取阳光
        if self.game_status != GameStatus::NotRunning {
            self.read_sun_value();
        }

        // 顶部面板
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("🌻 ZomKill v2.0");
        });

        // 中央功能区
        egui::CentralPanel::default().show(ctx, |ui| {
            // 游戏状态
            ui.group(|ui| {
                ui.heading("游戏状态");
                if self.game_status == GameStatus::Running {
                    ui.colored_label(egui::Color32::GREEN, "✓ 游戏已连接");
                    ui.label(format!("当前阳光: {}", self.current_sun));
                }
            });

            // 阳光修改
            ui.group(|ui| {
                ui.heading("🌞 阳光修改");
                ui.text_edit_singleline(&mut self.new_sun_value);
                if ui.button("🎯 修改阳光").clicked() {
                    self.write_sun_value();
                }
            });

            // 无冷却
            ui.group(|ui| {
                ui.heading("⚡ 植物卡槽");
                if ui.button("🚀 启动无冷却").clicked() {
                    self.toggle_no_cooldown();
                }
            });
        });

        // 每500ms刷新界面
        ctx.request_repaint_after(Duration::from_millis(500));
    }
}
```

#### 7. 中文字体支持

```rust
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 加载编译时嵌入的字体文件
    fonts.font_data.insert(
        "noto_sans_sc".to_owned(),
        egui::FontData::from_static(
            include_bytes!("../assets/fonts/NotoSansSC-Regular.ttf")
        ),
    );

    // 设置为默认字体
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "noto_sans_sc".to_owned());

    ctx.set_fonts(fonts);
}
```

**技术亮点**：
- 使用 `include_bytes!` 宏在编译时将字体打包进可执行文件
- 无需外部字体文件，程序开箱即用
- 使用 Noto Sans SC（思源黑体），支持所有中文字符

---

## 🔧 编译构建

### 环境准备

1. **安装 Rust**
   ```bash
   # 访问 https://rustup.rs/ 下载安装
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **添加 Windows 编译目标**（如果在非 Windows 系统编译）
   ```bash
   rustup target add x86_64-pc-windows-gnu
   ```

### 编译命令

```bash
# 克隆项目
git clone <repository_url>
cd zomkill

# 开发版本编译（包含调试信息）
cargo build

# 发布版本编译（优化编译，体积更小）
cargo build --release

# 运行程序
cargo run --release
```

### 交叉编译 Windows 版本（在 macOS/Linux 上）

```bash
# macOS 上编译 Windows 版本
cargo build --release --target x86_64-pc-windows-gnu

# 生成的文件位置
# target/x86_64-pc-windows-gnu/release/zomkill.exe
```

### 编译脚本（Windows）

项目包含 `build.bat` 脚本，双击即可编译：

```batch
@echo off
echo 正在编译 ZomKill...
cargo build --release
echo.
echo 编译完成！
echo 可执行文件位置: target\release\zomkill.exe
pause
```

---

## 🎓 技术总结

### 为什么选择 Rust？

1. **内存安全** - Rust 的所有权系统保证内存安全，避免 C++ 中的野指针、内存泄漏等问题
2. **零成本抽象** - 高级抽象不会带来性能损失
3. **现代工具链** - Cargo 包管理器简化依赖管理和构建流程
4. **优秀的 GUI 生态** - egui 是一个纯 Rust 的即时模式 GUI 库，简单易用

### Rust vs C++ 对比

| 特性 | C++ 版本 | Rust 版本 |
|------|---------|-----------|
| 代码行数 | ~200 行 | ~600 行（含 GUI） |
| 编译产物 | 控制台程序 | 图形界面程序 |
| 界面 | 纯文本菜单 | 现代化 GUI |
| 中文支持 | 依赖系统 | 内置字体 |
| 内存安全 | 手动管理 | 编译器保证 |
| 错误处理 | 返回值检查 | Result/Option |
| 依赖管理 | 手动配置 | Cargo 自动 |

### 内存修改原理

游戏数据存储在进程的内存空间中，修改器通过以下步骤修改数据：

```
1. 找到游戏进程 (FindWindowW + GetWindowThreadProcessId)
   ↓
2. 打开进程句柄 (OpenProcess)
   ↓
3. 读取基址获得中间地址 (ReadProcessMemory)
   ↓
4. 根据偏移量计算最终地址
   ↓
5. 读取/写入目标数据 (ReadProcessMemory/WriteProcessMemory)
   ↓
6. 关闭进程句柄 (CloseHandle)
```

### 关键 Windows API

| API 函数 | 功能 | 参数说明 |
|----------|------|----------|
| `FindWindowW` | 查找窗口 | 窗口类名、窗口标题 |
| `GetWindowThreadProcessId` | 获取进程ID | 窗口句柄 |
| `OpenProcess` | 打开进程 | 访问权限、进程ID |
| `ReadProcessMemory` | 读取内存 | 进程句柄、地址、缓冲区、大小 |
| `WriteProcessMemory` | 写入内存 | 进程句柄、地址、数据、大小 |
| `CloseHandle` | 关闭句柄 | 句柄 |

### 线程安全设计

```rust
// 使用 Arc 实现引用计数
// 使用 Mutex 保护共享状态
let no_cooldown_active: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

// 克隆 Arc，线程间共享所有权
let active_clone = Arc::clone(&no_cooldown_active);

// 在新线程中访问
std::thread::spawn(move || {
    while *active_clone.lock().unwrap() {
        // 执行无冷却操作
    }
});
```

---

## 🛠️ 高级技巧

### 1. 如何找到内存地址？

使用 **Cheat Engine (CE)** 工具：

1. 打开 CE，附加到游戏进程
2. 搜索当前阳光值（如 50）
3. 修改游戏中的阳光（如种植植物）
4. 在 CE 中搜索新值（如 0）
5. 重复多次，直到找到唯一地址
6. 右键 → "找出是什么改写了这个地址"
7. 分析汇编代码，找到基址和偏移


```

---

## ⚠️ 注意事项

### 法律与道德

- ✅ 本程序仅用于**个人学习和研究**
- ✅ 适用于**单机游戏的离线模式**
- ❌ 请勿用于**在线游戏或竞技对战**
- ❌ 请勿用于**商业用途或传播牟利**
- ❌ 请勿修改他人游戏影响体验

### 技术限制

1. **仅支持年度版** - 其他版本的内存地址可能不同
2. **需要管理员权限** - 某些系统需要以管理员身份运行
3. **杀毒软件误报** - 内存操作可能被杀软拦截，需添加信任
4. **游戏版本** - 其他游戏版本内存地址可能改变，请下载本站提供的游戏文件

---

**如果这篇文章对你有帮助，欢迎点赞、收藏、关注！** 👍


  [1]: https://xinux.top/usr/uploads/2025/11/3072738643.png
  [2]: https://xinux.top/usr/uploads/2025/11/1617654451.png
