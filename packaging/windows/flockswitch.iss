; Windows installer (Inno Setup 6). Installs for the current user only, so no
; administrator rights are needed: %LOCALAPPDATA%\Programs\flockswitch2.
; Built by the release workflow:
;   ISCC /DAppVersion=0.3.1 /DSourceDir=<files> /DIconFile=<ico> /DOutputDir=<dir> flockswitch.iss

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
AppId={{71155DB9-CBDB-4959-B0F8-2A9452A9C337}
AppName=flockswitch2
AppVersion={#AppVersion}
AppPublisher=WWhaleFe
AppPublisherURL=https://github.com/WWhaleFe/flockswitch2
AppSupportURL=https://github.com/WWhaleFe/flockswitch2/issues
DefaultDirName={autopf}\flockswitch2
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#OutputDir}
OutputBaseFilename=flockswitch2-setup-{#AppVersion}
SetupIconFile={#IconFile}
UninstallDisplayIcon={app}\flockswitch2.exe
UninstallDisplayName=flockswitch2
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
Name: "{autoprograms}\flockswitch2"; Filename: "{app}\flockswitch2.exe"
Name: "{autoprograms}\flockswitch2 Settings"; Filename: "{app}\flockswitch2.exe"; Parameters: "settings"
Name: "{autodesktop}\flockswitch2"; Filename: "{app}\flockswitch2.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\flockswitch2.exe"; Description: "{cm:LaunchProgram,flockswitch2}"; Flags: nowait postinstall skipifsilent
; A silent install is flockswitch updating itself: start the new version again, quietly.
Filename: "{app}\flockswitch2.exe"; Parameters: "tray --login"; Flags: nowait runasoriginaluser; Check: WizardSilent

[UninstallRun]
; Gives the mouse button back, removes start-at-login and the settings, quits the app.
Filename: "{app}\flockswitch2.exe"; Parameters: "uninstall"; Flags: runhidden waituntilterminated; RunOnceId: "flockswitch2-uninstall"

[Code]
// Upgrading: let the running flockswitch2 hand the button back and quit first.
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Code: Integer;
begin
  Result := '';
  if FileExists(ExpandConstant('{app}\flockswitch2.exe')) then
  begin
    Exec(ExpandConstant('{app}\flockswitch2.exe'), 'quit', '', SW_HIDE, ewWaitUntilTerminated, Code);
    // A settings window still open from this folder would keep the program file in use.
    Exec(ExpandConstant('{sys}\WindowsPowerShell\v1.0\powershell.exe'),
      '-NoProfile -NonInteractive -Command "Get-Process flockswitch2 -ErrorAction SilentlyContinue | ' +
      'Where-Object { $_.Path -eq ''' + ExpandConstant('{app}\flockswitch2.exe') + ''' } | Stop-Process -Force"',
      '', SW_HIDE, ewWaitUntilTerminated, Code);
  end;
end;
