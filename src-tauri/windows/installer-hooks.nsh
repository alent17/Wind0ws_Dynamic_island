; Isle Windows installer extensions.
; Keep this registry value in sync with services/settings.rs.
!define ISLE_AUTOSTART_KEY "Software\Microsoft\Windows\CurrentVersion\Run"
!define ISLE_AUTOSTART_VALUE "Wind0wsDynamicIsland"

!macro NSIS_HOOK_POSTUNINSTALL
  ; Tauri removes a value named after PRODUCTNAME; Isle itself uses the legacy
  ; Wind0wsDynamicIsland value, so remove that value as well.
  ${If} $UpdateMode <> 1
    DeleteRegValue HKCU "${ISLE_AUTOSTART_KEY}" "${ISLE_AUTOSTART_VALUE}"
  ${EndIf}
!macroend
