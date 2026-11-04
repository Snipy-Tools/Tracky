#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif

[Setup]
AppId={{4F6C2A91-8E3B-4D57-B1A0-7C9E5D2F3B68}
AppName=Tracky
AppVersion={#AppVersion}
AppPublisher=Snipy Tools
DefaultDirName={autopf}\Tracky
DefaultGroupName=Tracky
DisableProgramGroupPage=yes
UninstallDisplayIcon={app}\tracky.exe
SetupIconFile=src\assets\tracky.ico
OutputDir=dist
OutputBaseFilename=tracky-Windows-Setup
Compression=lzma2
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64compatible
CloseApplications=force
WizardStyle=modern

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; GroupDescription: "Additional shortcuts:"
Name: "autostart"; Description: "Start Tracky with &Windows"; GroupDescription: "Startup:"

[Files]
Source: "target\release\tracky.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Tracky"; Filename: "{app}\tracky.exe"
Name: "{autodesktop}\Tracky"; Filename: "{app}\tracky.exe"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "Tracky"; ValueData: """{app}\tracky.exe"" --background"; Flags: uninsdeletevalue; Tasks: autostart
; tells Tracky the autostart choice was already made here
Root: HKCU; Subkey: "Software\Tracky"; ValueType: dword; ValueName: "Initialized"; ValueData: 1; Flags: uninsdeletekey

[Run]
Filename: "{app}\tracky.exe"; Description: "Launch Tracky"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{sys}\taskkill.exe"; Parameters: "/f /im tracky.exe"; Flags: runhidden; RunOnceId: "KillTracky"

[UninstallDelete]
Type: filesandordirs; Name: "{localappdata}\Tracky"
