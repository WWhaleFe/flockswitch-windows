; Windows installer (Inno Setup 6). Installs for the current user only, so no
; administrator rights are needed: %LOCALAPPDATA%\Programs\flockswitch.
; Built by the release workflow:
;   ISCC /DAppVersion=0.3.1 /DSourceDir=<files> /DIconFile=<ico> /DOutputDir=<dir> flockswitch.iss

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
AppId={{8F2B6C1E-4D7A-4C2B-9E61-3A5F0D2C7B41}
AppName=flockswitch
AppVersion={#AppVersion}
AppPublisher=WWhaleFe
AppPublisherURL=https://github.com/WWhaleFe/flockswitch-windows
AppSupportURL=https://github.com/WWhaleFe/flockswitch-windows/issues
DefaultDirName={autopf}\flockswitch
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#OutputDir}
OutputBaseFilename=flockswitch-setup-{#AppVersion}
SetupIconFile={#IconFile}
UninstallDisplayIcon={app}\flockswitch.exe
UninstallDisplayName=flockswitch
LicenseFile={#SourceDir}\LICENSE
WizardStyle=modern
Compression=lzma2
SolidCompression=yes
; flockswitch is asked to quit in PrepareToInstall, so it can hand the button back.
CloseApplications=no

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
#if FileExists(AddBackslash(CompilerPath) + "Languages\Korean.isl")
Name: "ko"; MessagesFile: "compiler:Languages\Korean.isl"
#endif

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs

[Icons]
Name: "{autoprograms}\flockswitch"; Filename: "{app}\flockswitch.exe"
Name: "{autoprograms}\flockswitch Settings"; Filename: "{app}\flockswitch.exe"; Parameters: "settings"
Name: "{autodesktop}\flockswitch"; Filename: "{app}\flockswitch.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\flockswitch.exe"; Description: "{cm:LaunchProgram,flockswitch}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; Gives the mouse button back, removes start-at-login and the settings, quits the app.
Filename: "{app}\flockswitch.exe"; Parameters: "uninstall"; Flags: runhidden waituntilterminated; RunOnceId: "flockswitch-uninstall"

[Code]
// Upgrading: let the running flockswitch hand the button back and quit first.
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Code: Integer;
begin
  Result := '';
  if FileExists(ExpandConstant('{app}\flockswitch.exe')) then
    Exec(ExpandConstant('{app}\flockswitch.exe'), 'quit', '', SW_HIDE, ewWaitUntilTerminated, Code);
end;
