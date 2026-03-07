@echo off
setlocal enabledelayedexpansion

REM ============================================
REM 学校管理系统 - 一键构建发布脚本
REM 作用：构建前后端并整理到 newnewnew 发布目录
REM ============================================

set "ROOT_DIR=%~dp0.."
for %%I in ("%ROOT_DIR%") do set "ROOT_DIR=%%~fI"
set "OUTPUT_DIR=%ROOT_DIR%\newnewnew"

echo [INFO] 工作目录: %ROOT_DIR%
cd /d "%ROOT_DIR%" || (
  echo [ERROR] 无法进入项目根目录
  exit /b 1
)

echo [INFO] 开始构建前后端...
call npm run build
if errorlevel 1 (
  echo [ERROR] 构建失败，请先修复报错后重试
  exit /b 1
)

echo [INFO] 准备发布目录...
if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"
if not exist "%OUTPUT_DIR%\templates" mkdir "%OUTPUT_DIR%\templates"

if exist "%OUTPUT_DIR%\migrations" rmdir /s /q "%OUTPUT_DIR%\migrations"
if exist "%OUTPUT_DIR%\templates\permissions" rmdir /s /q "%OUTPUT_DIR%\templates\permissions"

mkdir "%OUTPUT_DIR%\migrations"
mkdir "%OUTPUT_DIR%\templates\permissions"

echo [INFO] 复制后端可执行文件...
copy /Y "%ROOT_DIR%\target\release\school-management-backend.exe" "%OUTPUT_DIR%\school-management-backend.exe" >nul
if errorlevel 1 (
  echo [ERROR] 复制 school-management-backend.exe 失败
  exit /b 1
)

copy /Y "%ROOT_DIR%\target\release\run_migration.exe" "%OUTPUT_DIR%\migration.exe" >nul
if errorlevel 1 (
  echo [ERROR] 复制 run_migration.exe 失败
  exit /b 1
)

echo [INFO] 复制迁移与模板文件...
xcopy "%ROOT_DIR%\backend\migrations\*" "%OUTPUT_DIR%\migrations\" /E /I /Y >nul
if errorlevel 1 (
  echo [ERROR] 复制 migrations 失败
  exit /b 1
)

xcopy "%ROOT_DIR%\backend\templates\permissions\*" "%OUTPUT_DIR%\templates\permissions\" /E /I /Y >nul
if errorlevel 1 (
  echo [ERROR] 复制 permissions 模板失败
  exit /b 1
)

echo [INFO] 复制环境变量文件（如存在）...
if exist "%ROOT_DIR%\.env" (
  copy /Y "%ROOT_DIR%\.env" "%OUTPUT_DIR%\.env" >nul
)

echo.
echo [SUCCESS] 打包完成！发布目录：%OUTPUT_DIR%
echo [SUCCESS] 关键文件：
echo   - school-management-backend.exe
echo   - migration.exe
echo   - static\ (前端产物)
echo   - migrations\
echo   - templates\permissions\
echo.
exit /b 0
