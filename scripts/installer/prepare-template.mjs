// Deterministic, reviewed changes on a pinned upstream Tauri NSIS template.
// Fail if upstream anchors change; never silently produce a partial patch.
import { readFile, writeFile } from 'node:fs/promises'
let source = await readFile('src-tauri/installer/upstream.nsi', 'utf8')
function replace(before, after) {
 if (!source.includes(before)) throw new Error('NSIS upstream anchor changed: '+before.slice(0,90))
 source = source.replace(before,after)
}
replace('{{#if installer_hooks}}\n!include "{{installer_hooks}}"\n{{/if}}','')
replace('${StrCase}\n${StrLoc}', '')
replace('; Installer pages, must be ordered as they appear', '!include "{{installer_hooks}}"\n\n; Installer pages, must be ordered as they appear')
replace('!define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive\n!insertmacro MUI_PAGE_WELCOME','Page custom LuneaWelcome')
const maintenanceStart = source.indexOf('Var ReinstallPageCheck')
const maintenanceEnd = source.indexOf('; 5. Choose install directory page')
if(maintenanceStart<0||maintenanceEnd<maintenanceStart)throw Error('Missing maintenance block')
source=source.slice(0,maintenanceStart)+'; In-place maintenance; never uninstall the previous version on a pre-install page.\nPage custom LuneaMaintenance\n\n'+source.slice(maintenanceEnd)
replace('!define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive\n!insertmacro MUI_PAGE_DIRECTORY','!define MUI_PAGE_CUSTOMFUNCTION_PRE LuneaDirectoryPre\n!define MUI_PAGE_CUSTOMFUNCTION_SHOW LuneaDirectoryShow\n!define MUI_PAGE_CUSTOMFUNCTION_LEAVE LuneaDirectoryLeave\n!insertmacro MUI_PAGE_DIRECTORY\nPage custom LuneaOptions LuneaOptionsLeave')
const start=source.indexOf('; 7. Installation page'),end=source.indexOf('; Uninstaller Pages')
if(start<0||end<start)throw Error('Missing page block')
source=source.slice(0,start)+`!define MUI_PAGE_CUSTOMFUNCTION_SHOW LuneaInstallShow
!insertmacro MUI_PAGE_INSTFILES
Page custom LuneaFinish LuneaFinishLeave

Function RunMainBinary
  nsis_tauri_utils::RunAsUser "$INSTDIR\\\${MAINBINARYNAME}.exe" ""
FunctionEnd

`+source.slice(end)
replace('Function .onInit\n','Function .onInit\n  Call LuneaInit\n')
replace('Function un.ConfirmShow ; Add add a `Delete app data` check box', 'Function un.ConfirmShow ; Keep user data opt-in\n  Call un.LuneaDarkPage')
replace('  SendMessage $DeleteAppDataCheckbox ${WM_SETFONT} $1 1', '  SendMessage $DeleteAppDataCheckbox ${WM_SETFONT} $1 1\n  System::Call \'uxtheme::SetWindowTheme(p $DeleteAppDataCheckbox, w "", w "")\'\n  SetCtlColors $DeleteAppDataCheckbox ${LUNEA_FG} ${LUNEA_BG}')
replace('!insertmacro MUI_UNPAGE_INSTFILES', '!define MUI_PAGE_CUSTOMFUNCTION_SHOW un.LuneaDarkPage\n!insertmacro MUI_UNPAGE_INSTFILES')
// Always restore a registered location, including /D and /UPDATE invocation.
replace('FunctionEnd\n\nSection WebView2','  Call LuneaValidateExisting\n  Call LuneaRejectDowngrade\nFunctionEnd\n\nSection WebView2')
replace('  ; Copy main executable\n  File "${MAINBINARYSRCPATH}"',`  ; Stage before touching an existing executable; NSIS verifies its embedded payload CRC.
  Call LuneaDirectoryLeave
  \${If} \${FileExists} "$INSTDIR\\\${MAINBINARYNAME}.new.exe"
    StrCpy $LuneaFailure "Найден файл незавершённой установки. Проверьте содержимое папки."
    Abort
  \${EndIf}
  StrCpy $LuneaStageOwned 1
  ClearErrors
  File "/oname=\${MAINBINARYNAME}.new.exe" "\${MAINBINARYSRCPATH}"
  IfErrors 0 +3
    StrCpy $LuneaFailure "Не удалось записать файлы. Проверьте доступ к папке и свободное место."
    Abort
  Call LuneaCommitBinary`)
replace('  ; Create start menu shortcut\n', '  DetailPrint "Создание выбранных ярлыков…"\n  ; Create start menu shortcut\n')
replace('    Call CreateOrUpdateStartMenuShortcut','    ${If} $LuneaStartMenu = 1\n      Call CreateOrUpdateStartMenuShortcut\n    ${EndIf}')
replace('  ; Create desktop shortcut for silent and passive installers\n  ; because finish page will be skipped\n  ${If} $PassiveMode = 1\n  ${OrIf} ${Silent}\n    Call CreateOrUpdateDesktopShortcut\n  ${EndIf}', '  ${If} $LuneaDesktop = 1\n    Call CreateOrUpdateDesktopShortcut\n  ${EndIf}')
replace('Function .onInstSuccess\n', 'Function .onInstSuccess\n  Delete "$INSTDIR\\${MAINBINARYNAME}.previous.exe"\n  StrCpy $LuneaCommitted 0\n  Push "Установка завершена успешно."\n  Call LuneaLog\n')
replace('  WriteUninstaller "$INSTDIR\\uninstall.exe"', '  ClearErrors\n  WriteUninstaller "$INSTDIR\\uninstall.exe"\n  IfErrors 0 +3\n    StrCpy $LuneaFailure "Не удалось создать программу удаления."\n    Abort\n  ClearErrors')
replace('  WriteRegDWORD SHCTX "${UNINSTKEY}" "NoRepair" "1"', '  WriteRegDWORD SHCTX "${UNINSTKEY}" "NoRepair" "1"\n  IfErrors 0 +3\n    StrCpy $LuneaFailure "Не удалось зарегистрировать приложение в Windows."\n    Abort')
replace('  !insertmacro CheckIfAppIsRunning "$INSTDIR\\${MAINBINARYNAME}.exe" "${PRODUCTNAME}"', '  Call LuneaValidateExisting\n  !insertmacro CheckIfAppIsRunning "$INSTDIR\\${MAINBINARYNAME}.exe" "${PRODUCTNAME}"\n  DetailPrint "Установка файлов…"')
await writeFile('src-tauri/installer/template.nsi',source)
console.log('Prepared pinned Tauri template with LUNÉA pages and in-place maintenance')

