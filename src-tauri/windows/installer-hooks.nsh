; Isle Windows installer extensions.
; Keep this registry value in sync with services/settings.rs.
!define ISLE_AUTOSTART_KEY "Software\Microsoft\Windows\CurrentVersion\Run"
!define ISLE_AUTOSTART_VALUE "Wind0wsDynamicIsland"

!macro NSIS_HOOK_POSTINSTALL
  ; Interactive installs let the user choose whether Isle starts with Windows.
  ; Passive/update installs preserve the existing preference.
  ${If} $PassiveMode <> 1
  ${AndIf} $UpdateMode <> 1
    MessageBox MB_ICONQUESTION|MB_YESNO|MB_DEFBUTTON1 \
      "是否让 Isle 在登录 Windows 后自动启动？$\r$\n$\r$\nLaunch Isle when Windows starts?" \
      IDYES isle_autostart_enable IDNO isle_autostart_disable

    isle_autostart_enable:
      WriteRegStr HKCU "${ISLE_AUTOSTART_KEY}" "${ISLE_AUTOSTART_VALUE}" '$\"$INSTDIR\${MAINBINARYNAME}.exe$\"'
      Goto isle_autostart_done

    isle_autostart_disable:
      DeleteRegValue HKCU "${ISLE_AUTOSTART_KEY}" "${ISLE_AUTOSTART_VALUE}"

    isle_autostart_done:
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Tauri removes a value named after PRODUCTNAME; Isle itself uses the legacy
  ; Wind0wsDynamicIsland value, so remove that value as well.
  ${If} $UpdateMode <> 1
    DeleteRegValue HKCU "${ISLE_AUTOSTART_KEY}" "${ISLE_AUTOSTART_VALUE}"
  ${EndIf}
!macroend
