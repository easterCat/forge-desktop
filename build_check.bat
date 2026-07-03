@echo off
cd /d C:\Users\fuhuo\Desktop\OH-WorkSpace\forge-desktop\src-tauri
call "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvarsall.bat" x64
cargo check 2>&1
