; Isle Windows installer extensions.
; Keep these registry values in sync with services/settings.rs.
!define ISLE_AUTOSTART_KEY "Software\Microsoft\Windows\CurrentVersion\Run"
!define ISLE_AUTOSTART_VALUE "Isle"
!define ISLE_LEGACY_AUTOSTART_VALUE "Wind0wsDynamicIsland"

!macro NSIS_HOOK_POSTUNINSTALL
  ; Remove both the current and legacy values. Tauri also removes the current
  ; product-name value, but keeping this explicit makes uninstall idempotent.
  ${If} $UpdateMode <> 1
    DeleteRegValue HKCU "${ISLE_AUTOSTART_KEY}" "${ISLE_AUTOSTART_VALUE}"
    DeleteRegValue HKCU "${ISLE_AUTOSTART_KEY}" "${ISLE_LEGACY_AUTOSTART_VALUE}"
  ${EndIf}
!macroend
