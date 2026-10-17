@echo off
title WowFriendBox (Lancement des 5 comptes WoW)
cd /d "%~dp0"
echo ===================================================
echo     Demarrage Automatique des 5 Comptes WoW
echo ===================================================
echo.
wowfriendbox.exe --cli --launch
pause
