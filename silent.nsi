SilentInstall silent
AutoCloseWindow true
ShowInstDetails hide
OutFile "Autotext2.exe"
Icon "resources/autotext128.ico"
Section ""
  SetOutPath "$TEMP\Autotext"
  File /r "*.dll"
  File "*.exe"
  ExecWait "$TEMP\Autotext\autotext.exe"
  RMDir /r "$TEMP\Autotext"
SectionEnd
