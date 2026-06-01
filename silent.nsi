SilentInstall silent
AutoCloseWindow true
ShowInstDetails hide
OutFile "Autotext.exe"
Icon "autotext.ico"
Section ""
  SetOutPath "$TEMP\Autotext"
  File /r "*.dll"
  File "*.exe"
  ExecWait "$TEMP\Autotext\autotext.exe"
  RMDir /r "$TEMP\Autotext"
SectionEnd
