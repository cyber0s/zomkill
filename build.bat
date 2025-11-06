@echo off
echo ========================================
echo    ZomKill v2.0 - 构建脚本
echo ========================================
echo.

echo 正在编译 Release 版本...
cargo build --release

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ========================================
    echo 编译成功！
    echo ========================================
    echo.
    echo 可执行文件位置: target\release\zomkill.exe
    echo.
    dir target\release\zomkill.exe
    echo.
    echo 按任意键退出...
    pause >nul
) else (
    echo.
    echo ========================================
    echo 编译失败，请检查错误信息
    echo ========================================
    echo.
    pause
)
