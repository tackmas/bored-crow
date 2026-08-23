!include "MUI2.nsh"
!include LogicLib.nsh
!include UnInstallLog.nsh

!define UninstLog "uninstall.log"
Var UninstLog

!define sc "$SYSDIR\sc.exe"
!define RUN_KEY "Software\Microsoft\Windows\CurrentVersion\Run"
!define BINARY_DIR "target\debug"
!define APP_NAME "Bored Crow"
!define REG_ROOT "HKCU"
!define REG_APP_PATH "SOFTWARE\${APP_NAME}"
!define UNINSTALL_PATH "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"
LangString UninstLogMissing ${LANG_ENGLISH} "${UninstLog} not found!$\r$\nUninstallation cannot proceed!"
!define AddItem "!insertmacro AddItem"
!define BackupFile "!insertmacro BackupFile" 
!define BackupFiles "!insertmacro BackupFiles" 
!define CopyFiles "!insertmacro CopyFiles"
!define CreateDirectory "!insertmacro CreateDirectory"
!define CreateShortcut "!insertmacro CreateShortcut"
!define File "!insertmacro File"
!define Rename "!insertmacro Rename"
!define RestoreFile "!insertmacro RestoreFile"    
!define RestoreFiles "!insertmacro RestoreFiles"
!define SetOutPath "!insertmacro SetOutPath"
!define WriteRegDWORD "!insertmacro WriteRegDWORD" 
!define WriteRegStr "!insertmacro WriteRegStr"
!define WriteUninstaller "!insertmacro WriteUninstaller"

RequestExecutionLevel admin

OutFile "installer.exe"
InstallDir "$PROGRAMFILES\Tackmas"

Section -openlogfile
  CreateDirectory "$INSTDIR"
  IfFileExists "$INSTDIR\${UninstLog}" +3
  FileOpen $UninstLog "$INSTDIR\${UninstLog}" w
  Goto +4
  SetFileAttributes "$INSTDIR\${UninstLog}" NORMAL
  FileOpen $UninstLog "$INSTDIR\${UninstLog}" a
  FileSeek $UninstLog 0 END
SectionEnd

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

Section "Install"
  ${SetOutPath} "$INSTDIR"

  ${File} "${BINARY_DIR}\daemon.exe" daemon.exe
  ${File} "${BINARY_DIR}\desktop.exe" desktop.exe
  ${File} "${BINARY_DIR}\can_uninstall.exe" can_uninstall.exe
  ${File} "${BINARY_DIR}\restarter.exe" restarter.exe

  ExecWait '"$INSTDIR\daemon.exe" register-service'
  IfErrors +1 +2
    MessageBox MB_ABORTRETRYIGNORE '"daemon.exe register-service" returned an error'

  ExecWait '${sc} start "${APP_NAME}"'
  IfErrors +1 +2
    MessageBox MB_ABORTRETRYIGNORE '"sc start ${APP_NAME}" returned an error'

  ExecWait '${sc} failure "${APP_NAME}" reset= 1 actions= restart/1000'
  IfErrors +1 +2
    MessageBox MB_ABORTRETRYIGNORE '"sc failure "${APP_NAME}"" returned an error'

  ${WriteRegStr} ${REG_ROOT} "${REG_APP_PATH}" "Install Directory" "$INSTDIR"
  ${WriteRegStr} ${REG_ROOT} "${UNINSTALL_PATH}" "UninstallString" "$INSTDIR\uninstall.exe"
  ${WriteUninstaller} "$INSTDIR\uninstall.exe"
SectionEnd


; Uninstaller
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

Function un.onInit 
  ExecWait '"$INSTDIR\can_uninstall.exe"'
  IfErrors +1 Done
    MessageBox MB_OK|MB_ICONEXCLAMATION "Can not uninstall"
    Abort

  Sleep 2000

  Done:
FunctionEnd

Section Uninstall
  MessageBox MB_OK "Can uninstall. Uninstalling"

  ;Can't uninstall if uninstall log is missing!
  IfFileExists "$INSTDIR\${UninstLog}" +3
    MessageBox MB_OK|MB_ICONSTOP "$(UninstLogMissing)"
      Abort

  ClearErrors

  ;ExecWait '${sc} stop "${APP_NAME}"'
  ;IfErrors +1 +2
    ;MessageBox MB_ABORTRETRYIGNORE '"sc stop ${APP_NAME}" returned an error'

  Sleep 500

  ExecWait '${sc} delete "${APP_NAME}"'
  IfErrors +1 +2
    MessageBox MB_ABORTRETRYIGNORE '"sc delete ${APP_NAME}" returned an error'
 
  Push $R0
  Push $R1
  Push $R2
  SetFileAttributes "$INSTDIR\${UninstLog}" NORMAL
  FileOpen $UninstLog "$INSTDIR\${UninstLog}" r
  StrCpy $R1 -1
 
  GetLineCount:
    ClearErrors
    FileRead $UninstLog $R0
    IntOp $R1 $R1 + 1
    StrCpy $R0 $R0 -2
    Push $R0   
    IfErrors 0 GetLineCount
 
  Pop $R0
 
  LoopRead:
    StrCmp $R1 0 LoopDone
    Pop $R0
 
    IfFileExists "$R0\*.*" 0 +3
      RMDir $R0  #is dir
    Goto +9
    IfFileExists $R0 0 +3
      Delete $R0 #is file
    Goto +6
    StrCmp $R0 "${REG_ROOT} ${REG_APP_PATH}" 0 +3
      DeleteRegKey ${REG_ROOT} "${REG_APP_PATH}" #is Reg Element
    Goto +3
    StrCmp $R0 "${REG_ROOT} ${UNINSTALL_PATH}" 0 +2
      DeleteRegKey ${REG_ROOT} "${UNINSTALL_PATH}" #is Reg Element
 
    IntOp $R1 $R1 - 1
    Goto LoopRead
  LoopDone:
  FileClose $UninstLog
  Delete "$INSTDIR\${UninstLog}"
  RMDir "$INSTDIR"
  Pop $R2
  Pop $R1
  Pop $R0
 
  ;Remove registry keys
    ;DeleteRegKey ${REG_ROOT} "${REG_APP_PATH}"
    ;DeleteRegKey ${REG_ROOT} "${UNINSTALL_PATH}"
SectionEnd