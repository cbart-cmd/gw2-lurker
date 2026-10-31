@echo off
rem Copia gw2_lurker.dll a la carpeta addons de Guild Wars 2 con doble clic.
rem Argumentos opcionales: install.bat -Build   (tests + compilacion antes de copiar)
rem                        install.bat -DryRun  (solo dice que haria)
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1" %*
echo.
pause
