!define APP_NAME "MD Viewer"
!define APP_VERSION "0.6.0"
!define APP_EXE "md-viewer.exe"
!define APP_DIR "MD Viewer"

Name "${APP_NAME} ${APP_VERSION}"
OutFile "..\dist\MDViewerSetup-${APP_VERSION}.exe"
InstallDir "$LOCALAPPDATA\Programs\${APP_DIR}"
RequestExecutionLevel user
Unicode true
Icon "md-viewer.ico"
UninstallIcon "md-viewer.ico"

Section "Install"
    SetOutPath "$INSTDIR"
    File "..\target\release\${APP_EXE}"
    File "md-viewer.ico"
    CreateDirectory "$SMPROGRAMS\${APP_DIR}"
    CreateShortcut "$SMPROGRAMS\${APP_DIR}\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}" "" "$INSTDIR\md-viewer.ico"
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    WriteRegStr HKCU "Software\Classes\.md\OpenWithProgids" "MDViewer.Markdown" ""
    WriteRegStr HKCU "Software\Classes\.markdown\OpenWithProgids" "MDViewer.Markdown" ""
    WriteRegStr HKCU "Software\Classes\MDViewer.Markdown\DefaultIcon" "" "$INSTDIR\md-viewer.ico"
    WriteRegStr HKCU "Software\Classes\MDViewer.Markdown\shell\open\command" "" '"$INSTDIR\${APP_EXE}" "%1"'
    WriteRegStr HKCU "Software\Classes\MDViewer.Markdown\shell\open\FriendlyAppName" "" "${APP_NAME}"

    WriteRegStr HKCU "Software\MDViewer\Capabilities" "ApplicationName" "${APP_NAME}"
    WriteRegStr HKCU "Software\MDViewer\Capabilities" "ApplicationDescription" "View Markdown files"
    WriteRegStr HKCU "Software\MDViewer\Capabilities\FileAssociations" ".md" "MDViewer.Markdown"
    WriteRegStr HKCU "Software\MDViewer\Capabilities\FileAssociations" ".markdown" "MDViewer.Markdown"
    WriteRegStr HKCU "Software\RegisteredApplications" "MD Viewer" "Software\MDViewer\Capabilities"

    System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, i 0, i 0)'
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\${APP_EXE}"
    Delete "$INSTDIR\md-viewer.ico"
    Delete "$APPDATA\${APP_DIR}\settings.txt"
    RMDir "$APPDATA\${APP_DIR}"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"
    Delete "$SMPROGRAMS\${APP_DIR}\${APP_NAME}.lnk"
    RMDir "$SMPROGRAMS\${APP_DIR}"

    DeleteRegValue HKCU "Software\Classes\.md\OpenWithProgids" "MDViewer.Markdown"
    DeleteRegValue HKCU "Software\Classes\.markdown\OpenWithProgids" "MDViewer.Markdown"
    DeleteRegKey HKCU "Software\Classes\MDViewer.Markdown"
    DeleteRegKey HKCU "Software\MDViewer\Capabilities"
    DeleteRegValue HKCU "Software\RegisteredApplications" "MD Viewer"

    System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, i 0, i 0)'
SectionEnd
