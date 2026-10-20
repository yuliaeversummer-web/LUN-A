; Native NSIS UI. All coordinates are dialog units for Windows DPI scaling.
; Static artwork intentionally respects reduced motion without OS polling/timers.
!define LUNEA_BG "0d1216"
!define LUNEA_FG "f5edf3"
!define LUNEA_ACCENT "fe007f"
!define LUNEA_SECONDARY "e19cbb"
; Alternative token values for future builds: 55d5fc / f1ca7c / e7272d.
!define MUI_BGCOLOR "${LUNEA_BG}"
!define MUI_TEXTCOLOR "${LUNEA_FG}"
!define MUI_CUSTOMFUNCTION_GUIINIT LuneaGUIInit
!define MUI_CUSTOMFUNCTION_UNGUIINIT un.LuneaGUIInit
!define MUI_INSTFILESPAGE_COLORS "${LUNEA_SECONDARY} ${LUNEA_BG}"
!define MUI_ABORTWARNING
!define MUI_ABORTWARNING_TEXT "Прервать подготовку установки LUNÉA? Установленные файлы ещё не изменены."
!define MUI_DIRECTORYPAGE_TEXT_TOP "Выберите папку для LUNÉA. Для существующей установки используется её текущая папка."
!define MUI_DIRECTORYPAGE_VERIFYONLEAVE
!define MUI_FINISHPAGE_NOAUTOCLOSE
!define MUI_HEADERIMAGE_BITMAP_STRETCH FitControl
!define MUI_HEADERIMAGE_RIGHT
!define MUI_HEADERIMAGE_UNBITMAP_STRETCH FitControl
BrandingText " "
SetFont "Segoe UI" 10
InstallColors ${LUNEA_SECONDARY} ${LUNEA_BG}
ShowInstDetails show

Var LuneaDialog
Var LuneaImage
Var LuneaBitmap
Var LuneaFont
Var LuneaControl
Var LuneaDesktopControl
Var LuneaMenuControl
Var LuneaRunControl
Var LuneaDesktop
Var LuneaStartMenu
Var LuneaRun
Var LuneaExisting
Var LuneaExistingVersion
Var LuneaLogPath
Var LuneaFailure
Var LuneaCommitted
Var LuneaStageOwned
Var LuneaLaunchButton
Var LuneaNextExpanded

Function LuneaGUIInit
  ; Windows 11 native dark caption; failure on older Windows is harmless.
  System::Call 'dwmapi::DwmSetWindowAttribute(p $HWNDPARENT, i 20, *i 1, i 4)'
  SetCtlColors $HWNDPARENT ${LUNEA_FG} ${LUNEA_BG}
  GetDlgItem $LuneaControl $HWNDPARENT 1028
  SetCtlColors $LuneaControl ${LUNEA_SECONDARY} ${LUNEA_BG}
  GetDlgItem $LuneaControl $HWNDPARENT 1256
  SetCtlColors $LuneaControl ${LUNEA_SECONDARY} ${LUNEA_BG}
FunctionEnd

Function un.LuneaGUIInit
  System::Call 'dwmapi::DwmSetWindowAttribute(p $HWNDPARENT, i 20, *i 1, i 4)'
  SetCtlColors $HWNDPARENT ${LUNEA_FG} ${LUNEA_BG}
FunctionEnd

Function LuneaWideNext
  ; Resize from the actual themed/DPI-scaled button rectangle, no pixel constants.
  Push $0
  Push $1
  Push $2
  Push $3
  Push $4
  Push $5
  GetDlgItem $0 $HWNDPARENT 1
  System::Alloc 16
  Pop $1
  System::Call 'user32::GetWindowRect(p r0, p r1)'
  System::Call 'user32::MapWindowPoints(p 0, p $HWNDPARENT, p r1, i 2)'
  System::Call '*$1(i .r2, i .r3, i .r4, i .r5)'
  IntOp $5 $5 - $3
  IntOp $4 $4 - $2
  ${If} $LuneaNextExpanded = 1
    IntOp $4 $4 / 2
    IntOp $2 $2 + $4
    StrCpy $LuneaNextExpanded 0
  ${Else}
    IntOp $2 $2 - $4
    IntOp $4 $4 * 2
    StrCpy $LuneaNextExpanded 1
  ${EndIf}
  System::Call 'user32::SetWindowPos(p r0, p 0, i r2, i r3, i r4, i r5, i 0x14)'
  System::Free $1
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

!macro LuneaLabel X Y W H TEXT COLOR
  ${NSD_CreateLabel} ${X} ${Y} ${W} ${H} "${TEXT}"
  Pop $LuneaControl
  SetCtlColors $LuneaControl ${COLOR} ${LUNEA_BG}
!macroend

Function LuneaInit
  StrCpy $LuneaDesktop 1
  StrCpy $LuneaStartMenu 1
  ; Opt-in by default; no unexpected application launch on Finish.
  StrCpy $LuneaRun 0
  StrCpy $LuneaCommitted 0
  StrCpy $LuneaStageOwned 0
  StrCpy $WixMode 0
  StrCpy $LuneaLogPath "$TEMP\Lunea-setup-${VERSION}.log"
  StrCpy $LuneaFailure "Установка не завершена. Подробности доступны в окне установки."
  CreateFont $LuneaFont "Segoe UI" 17 600
  Push "Начало установки ${VERSION}."
  Call LuneaLog
FunctionEnd

Function LuneaLog
  Exch $0
  Push $1
  ClearErrors
  FileOpen $1 $LuneaLogPath a
  ${IfNot} ${Errors}
    FileWrite $1 "$0$\r$\n"
    FileClose $1
  ${EndIf}
  Pop $1
  Pop $0
FunctionEnd

Function LuneaValidateExisting
  ReadRegStr $LuneaExisting SHCTX "${MANUPRODUCTKEY}" ""
  ReadRegStr $LuneaExistingVersion SHCTX "${UNINSTKEY}" "DisplayVersion"
  ${If} $LuneaExisting != ""
    ; One registered installation, even if /D points elsewhere.
    StrCpy $INSTDIR $LuneaExisting
  ${EndIf}
FunctionEnd

Function LuneaRejectDowngrade
  ReadRegStr $LuneaControl SHCTX "${UNINSTKEY}" "UninstallString"
  ${If} $LuneaControl != ""
  ${AndIf} $LuneaExisting == ""
    ${IfNot} ${Silent}
      MessageBox MB_ICONSTOP "Обнаружена установка с повреждённой регистрацией пути. Восстановите или удалите её через Windows перед новой установкой."
    ${EndIf}
    SetErrorLevel 1
    Abort
  ${EndIf}
  ${If} $LuneaExistingVersion != ""
    nsis_tauri_utils::SemverCompare "${VERSION}" $LuneaExistingVersion
    Pop $LuneaControl
    ${If} $LuneaControl = -1
      ${IfNot} ${Silent}
        MessageBox MB_ICONSTOP "Установлена более новая версия LUNÉA ($LuneaExistingVersion). Понижение до ${VERSION} отменено."
      ${EndIf}
      Push "Отклонено понижение версии."
      Call LuneaLog
      SetErrorLevel 1
      Abort
    ${EndIf}
  ${EndIf}
FunctionEnd

Function LuneaArt
  InitPluginsDir
  File "/oname=$PLUGINSDIR\lunea-mascot.bmp" "${__FILEDIR__}\assets\mascot-panel.bmp"
  ${NSD_CreateBitmap} 0u 8u 150u 167u ""
  Pop $LuneaImage
  ${NSD_SetStretchedBitmap} $LuneaImage "$PLUGINSDIR\lunea-mascot.bmp" $LuneaBitmap
FunctionEnd

Function LuneaWelcome
  Call SkipIfPassive
  nsDialogs::Create 1044
  Pop $LuneaDialog
  ${If} $LuneaDialog == error
    Abort
  ${EndIf}
  SetCtlColors $LuneaDialog ${LUNEA_FG} ${LUNEA_BG}
  Call LuneaArt
  !insertmacro LuneaLabel 160u 14u 158u 14u "L U N É A  /  ${VERSION}" ${LUNEA_SECONDARY}
  !insertmacro LuneaLabel 160u 43u 158u 43u "Добро пожаловать$\r$\nв LUNÉA" ${LUNEA_FG}
  SendMessage $LuneaControl ${WM_SETFONT} $LuneaFont 1
  !insertmacro LuneaLabel 160u 95u 150u 22u "Твой звук. Твоя атмосфера." ${LUNEA_SECONDARY}
  !insertmacro LuneaLabel 160u 126u 150u 45u "Звук Windows, музыка и Discord — в одном пространстве.$\r$\n$\r$\nУстановка только для вас, без прав администратора." ${LUNEA_FG}
  GetDlgItem $LuneaControl $HWNDPARENT 1
  Call LuneaWideNext
  SendMessage $LuneaControl ${WM_SETTEXT} 0 "STR:Начать установку"
  nsDialogs::Show
  Call LuneaWideNext
  ${NSD_FreeBitmap} $LuneaBitmap
FunctionEnd

Function LuneaMaintenance
  Call LuneaValidateExisting
  ${If} $LuneaExisting == ""
    Abort
  ${EndIf}
  Call SkipIfPassive
  !insertmacro MUI_HEADER_TEXT "LUNÉA уже установлена" "Обновление в существующей папке"
  nsDialogs::Create 1018
  Pop $LuneaDialog
  SetCtlColors $LuneaDialog ${LUNEA_FG} ${LUNEA_BG}
  !insertmacro LuneaLabel 0u 6u 100% 28u "Установлена версия $LuneaExistingVersion. Пакет: ${VERSION}." ${LUNEA_SECONDARY}
  !insertmacro LuneaLabel 0u 39u 100% 30u "$LuneaExisting" ${LUNEA_FG}
  !insertmacro LuneaLabel 0u 77u 100% 60u "Продолжить обновление или восстановление этой установки.$\r$\nНастройки, темы и назначения горячих клавиш сохраняются.$\r$\nДо начала копирования можно безопасно отменить установку.$\r$\nДля удаления используйте «Установленные приложения» Windows." ${LUNEA_FG}
  nsDialogs::Show
FunctionEnd

Function LuneaDirectoryPre
  Call SkipIfPassive
  Call LuneaValidateExisting
FunctionEnd

Function LuneaDirectoryShow
  Call LuneaDarkPage
  ${If} $LuneaExisting != ""
    FindWindow $LuneaDialog "#32770" "" $HWNDPARENT
    GetDlgItem $LuneaControl $LuneaDialog 1019
    EnableWindow $LuneaControl 0
    GetDlgItem $LuneaControl $LuneaDialog 1001
    EnableWindow $LuneaControl 0
  ${EndIf}
FunctionEnd

Function LuneaDirectoryLeave
  ; Never overwrite an unregistered directory containing an unrelated binary.
  ${If} $LuneaExisting == ""
    ${If} ${FileExists} "$INSTDIR\${MAINBINARYNAME}.exe"
    ${OrIf} ${FileExists} "$INSTDIR\uninstall.exe"
    ${IfNot} ${Silent}
      MessageBox MB_ICONEXCLAMATION "В этой папке уже есть lunea.exe без регистрации установщика. Выберите другую папку."
    ${EndIf}
    StrCpy $LuneaFailure "Папка содержит незарегистрированный lunea.exe; перезапись запрещена."
    Abort
    ${EndIf}
  ${EndIf}
FunctionEnd

Function LuneaOptions
  Call SkipIfPassive
  !insertmacro MUI_HEADER_TEXT "Последние штрихи" "Настройте ярлыки и запуск после установки"
  nsDialogs::Create 1018
  Pop $LuneaDialog
  SetCtlColors $LuneaDialog ${LUNEA_FG} ${LUNEA_BG}
  ${NSD_CreateCheckbox} 4u 14u 285u 15u "Создать ярлык на рабочем столе"
  Pop $LuneaDesktopControl
  SetCtlColors $LuneaDesktopControl ${LUNEA_FG} ${LUNEA_BG}
  System::Call 'uxtheme::SetWindowTheme(p $LuneaDesktopControl, w "", w "")'
  ${NSD_SetState} $LuneaDesktopControl $LuneaDesktop
  ${NSD_CreateCheckbox} 4u 43u 285u 15u "Создать ярлык в меню «Пуск»"
  Pop $LuneaMenuControl
  SetCtlColors $LuneaMenuControl ${LUNEA_FG} ${LUNEA_BG}
  System::Call 'uxtheme::SetWindowTheme(p $LuneaMenuControl, w "", w "")'
  ${NSD_SetState} $LuneaMenuControl $LuneaStartMenu
  ${NSD_CreateCheckbox} 4u 72u 285u 15u "Запустить LUNÉA после завершения"
  Pop $LuneaRunControl
  SetCtlColors $LuneaRunControl ${LUNEA_FG} ${LUNEA_BG}
  System::Call 'uxtheme::SetWindowTheme(p $LuneaRunControl, w "", w "")'
  ${NSD_SetState} $LuneaRunControl $LuneaRun
  !insertmacro LuneaLabel 4u 103u 285u 36u "Автозапуск Windows не включается.$\r$\nВо время короткого копирования файлов закрытие будет недоступно — дождитесь результата." ${LUNEA_SECONDARY}
  GetDlgItem $LuneaControl $HWNDPARENT 1
  SendMessage $LuneaControl ${WM_SETTEXT} 0 "STR:Установить"
  nsDialogs::Show
FunctionEnd

Function LuneaOptionsLeave
  ${NSD_GetState} $LuneaDesktopControl $LuneaDesktop
  ${NSD_GetState} $LuneaMenuControl $LuneaStartMenu
  ${NSD_GetState} $LuneaRunControl $LuneaRun
FunctionEnd

Function LuneaInstallShow
  Call LuneaDarkPage
  GetDlgItem $LuneaControl $HWNDPARENT 2
  EnableWindow $LuneaControl 0
  !insertmacro MUI_HEADER_TEXT "Создаём ваше пространство" "Настоящий прогресс копирования NSIS. Дождитесь завершения."
  Push "Подготовка компонентов и установка файлов."
  Call LuneaLog
FunctionEnd

Function LuneaCommitBinary
  !ifdef LUNEA_QA_UI
    ; Test-only observation window after actual payload extraction, not simulated progress.
    DetailPrint "QA: файлы распакованы; пауза для снимка настоящего окна NSIS."
    Sleep 10000
  !endif
  ; Keep the previous executable until .onInstSuccess; no uninstall-on-update.
  ${If} ${FileExists} "$INSTDIR\${MAINBINARYNAME}.previous.exe"
    StrCpy $LuneaFailure "Найден резервный файл предыдущей установки. Закройте установщик и проверьте папку; файлы не удалены."
    Abort
  ${EndIf}
  ${If} ${FileExists} "$INSTDIR\${MAINBINARYNAME}.exe"
    ClearErrors
    Rename "$INSTDIR\${MAINBINARYNAME}.exe" "$INSTDIR\${MAINBINARYNAME}.previous.exe"
    ${If} ${Errors}
      StrCpy $LuneaFailure "Не удалось обновить файл LUNÉA. Закройте приложение и повторите установку."
      Abort
    ${EndIf}
  ${EndIf}
  StrCpy $LuneaCommitted 1
  ClearErrors
  Rename "$INSTDIR\${MAINBINARYNAME}.new.exe" "$INSTDIR\${MAINBINARYNAME}.exe"
  ${If} ${Errors}
    StrCpy $LuneaFailure "Не удалось завершить замену файла LUNÉA."
    Abort
  ${EndIf}
FunctionEnd

Function .onInstFailed
  ${If} $LuneaCommitted = 1
  ${AndIf} ${FileExists} "$INSTDIR\${MAINBINARYNAME}.previous.exe"
    Delete "$INSTDIR\${MAINBINARYNAME}.exe"
    Rename "$INSTDIR\${MAINBINARYNAME}.previous.exe" "$INSTDIR\${MAINBINARYNAME}.exe"
  ${EndIf}
  ${If} $LuneaStageOwned = 1
    Delete "$INSTDIR\${MAINBINARYNAME}.new.exe"
  ${EndIf}
  ${If} $LuneaExistingVersion != ""
    WriteRegStr SHCTX "${UNINSTKEY}" "DisplayVersion" $LuneaExistingVersion
  ${EndIf}
  ${If} $LuneaCommitted = 1
  ${AndIf} $LuneaExisting == ""
    Delete "$INSTDIR\${MAINBINARYNAME}.exe"
    Delete "$INSTDIR\uninstall.exe"
    DeleteRegKey SHCTX "${UNINSTKEY}"
    DeleteRegValue SHCTX "${MANUPRODUCTKEY}" ""
  ${EndIf}
  Push $LuneaFailure
  Call LuneaLog
  ${IfNot} ${Silent}
    MessageBox MB_ICONSTOP "$LuneaFailure$\r$\n$\r$\nМожно закрыть установщик и повторить установку после устранения причины.$\r$\nДиагностический лог: $LuneaLogPath"
  ${EndIf}
  SetErrorLevel 1
FunctionEnd

Function LuneaFinish
  !ifndef LUNEA_QA_UI
    Call SkipIfPassive
  !endif
  nsDialogs::Create 1044
  Pop $LuneaDialog
  SetCtlColors $LuneaDialog ${LUNEA_FG} ${LUNEA_BG}
  Call LuneaArt
  !insertmacro LuneaLabel 160u 14u 158u 14u "L U N É A  /  ${VERSION}" ${LUNEA_SECONDARY}
  !insertmacro LuneaLabel 160u 43u 158u 42u "Всё готово.$\r$\nВаш звук — рядом." ${LUNEA_FG}
  SendMessage $LuneaControl ${WM_SETFONT} $LuneaFont 1
  !insertmacro LuneaLabel 160u 97u 150u 28u "LUNÉA успешно установлена.$\r$\nCtrl+D открывает компактную панель." ${LUNEA_SECONDARY}
  ${NSD_CreateButton} 160u 138u 148u 20u "Запустить LUNÉA"
  Pop $LuneaLaunchButton
  ${NSD_OnClick} $LuneaLaunchButton LuneaLaunch
  ${If} $LuneaRun = 1
    !insertmacro LuneaLabel 160u 165u 150u 18u "Также запустится после «Готово»." ${LUNEA_SECONDARY}
  ${EndIf}
  GetDlgItem $LuneaControl $HWNDPARENT 1
  SendMessage $LuneaControl ${WM_SETTEXT} 0 "STR:Готово"
  GetDlgItem $LuneaControl $HWNDPARENT 3
  EnableWindow $LuneaControl 0
  nsDialogs::Show
  ${NSD_FreeBitmap} $LuneaBitmap
FunctionEnd

Function LuneaLaunch
  Pop $LuneaControl
  Call RunMainBinary
  StrCpy $LuneaRun 0
  EnableWindow $LuneaLaunchButton 0
FunctionEnd

Function LuneaFinishLeave
  ${If} $LuneaRun = 1
    Call RunMainBinary
  ${EndIf}
FunctionEnd

Function .onGUIEnd
  ${If} $LuneaStageOwned = 1
    Delete "$INSTDIR\${MAINBINARYNAME}.new.exe"
  ${EndIf}
  System::Call 'gdi32::DeleteObject(p $LuneaFont)'
FunctionEnd

!macro LuneaDarkPage PREFIX
Function ${PREFIX}LuneaDarkPage
  Push $0
  Push $1
  FindWindow $0 "#32770" "" $HWNDPARENT
  SetCtlColors $0 ${LUNEA_FG} ${LUNEA_BG}
  GetDlgItem $1 $0 1020
  System::Call 'uxtheme::SetWindowTheme(p r1, w "", w "")'
  System::Call 'user32::GetWindow(p r0, i 5) p .r1'
  ${DoWhile} $1 P<> 0
    SetCtlColors $1 ${LUNEA_FG} ${LUNEA_BG}
    System::Call 'user32::GetWindow(p r1, i 2) p .r1'
  ${Loop}
  Pop $1
  Pop $0
FunctionEnd
!macroend
!insertmacro LuneaDarkPage ""
!insertmacro LuneaDarkPage "un."
