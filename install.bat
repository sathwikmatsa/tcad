:: Batch script for installing TCAD in Windows.
@ECHO OFF
setlocal enabledelayedexpansion

:: Dependency check
WHERE wget >nul 2>&1
IF %ERRORLEVEL% NEQ 0 GOTO DEP_ERROR

SET TCAD_DIR=%USERPROFILE%\.bin\tcad

:: add tcad directory to path
for /f "usebackq tokens=2,*" %%A in (`reg query HKCU\Environment /v PATH`) do set user_path=%%B
SETX PATH "%user_path%;%TCAD_DIR%"

:: Check for tcad.exe
IF EXIST tcad.exe (
    MKDIR %TCAD_DIR%
    COPY /Y tcad.exe %TCAD_DIR%
    COPY /Y toast.ps1 %TCAD_DIR%
    CHDIR %TCAD_DIR%

    call :SET_TCLOUD_URL
    SET /P DOWNLOAD_DIR="Enter Download directory (default: %USERPROFILE%\Downloads)"
    IF !DOWNLOAD_DIR!.==. SET DOWNLOAD_DIR=%USERPROFILE%\Downloads
    call :CHECK_DOWNLOAD_DIR

    :: Create .env file
    TYPE nul >.env
    ECHO TCLOUD_URL="!TCLOUD_URL!" > .env
    ECHO DOWNLOAD_DIR='!DOWNLOAD_DIR!' >> .env
    ECHO LOG_DIR='%TCAD_DIR%' >> .env

    :: Create VisualBasic script to execute tcad without popping window
    TYPE nul >_tcad.vbs
    SET VBS="CreateObject("Wscript.Shell").Run "%TCAD_DIR%\tcad.exe", 0, True"
    ECHO !VBS:~1,-1! > _tcad.vbs

    :: Schedule tcad
    SCHTASKS /DELETE /TN "TCAD" /F >nul 2>&1
    SCHTASKS /CREATE /SC MINUTE /MO 5 /TN "TCAD" /TR "wscript.exe %TCAD_DIR%\_tcad.vbs" /F

    ECHO.---*---*---
    ECHO.Successfully installed TCAD!
    ECHO.Add torrents at !TCLOUD_URL!
    ECHO.They will be downloaded to !DOWNLOAD_DIR!
) ELSE (
    ECHO.Error: `tcad.exe` not found in current directory.
    ECHO.Make sure you run install.bat from release directory.
)
GOTO:eof

:SET_TCLOUD_URL
    SET /P TCLOUD_URL="Enter TCLOUD URL:"
    IF !TCLOUD_URL!.==. goto SET_TCLOUD_URL
    IF !TCLOUD_URL:~-1!==/ SET TCLOUD_URL=!TCLOUD_URL:~0,-1!
    exit /b

:CHECK_DOWNLOAD_DIR
    IF NOT EXIST !DOWNLOAD_DIR! (
        SET /P DOWNLOAD_DIR="Error, Enter download directory:"    
        goto CHECK_DOWNLOAD_DIR
    ) else (
        exit /b
    )

:DEP_ERROR
    ECHO.Error: `wget` not found. Please install via "choco install wget". 
    ECHO.https://chocolatey.org/packages/Wget

