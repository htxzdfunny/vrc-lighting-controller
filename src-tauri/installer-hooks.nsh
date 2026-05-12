!macro NSIS_HOOK_PREINSTALL
  ; --- VC++ Redistributable check ---
  Push $0
  SetRegView 64
  ClearErrors
  ReadRegDWORD $0 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" "Installed"
  SetRegView 32
  IfErrors vcredist_missing 0
  IntCmp $0 1 vcredist_ok vcredist_missing vcredist_ok

vcredist_missing:
  Pop $0
  MessageBox MB_ICONSTOP|MB_OK "缺少 Microsoft Visual C++ 2015-2022 Redistributable (x64)。$\r$\n请先安装后再继续：$\r$\nhttps://aka.ms/vs/17/release/vc_redist.x64.exe"
  Abort

vcredist_ok:
  Pop $0
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; --- WebView2 Runtime check & install ---
  ; Tauri's built-in detection can be fooled by Edge registering the same
  ; registry key. We do our own check here: probe for the standalone
  ; WebView2 Runtime AND the per-user "EBWebView" folder that a functional
  ; WebView2 produces. If neither looks right, run the embedded bootstrapper.
  Push $0
  Push $1

  ; 1) Check the machine-wide pv key (same one Tauri checks)
  SetRegView 64
  ClearErrors
  ReadRegStr $0 HKLM "SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" "pv"
  SetRegView 32
  IfErrors wv2_not_found 0
  StrCmp $0 "" wv2_not_found 0

  ; 2) Verify the runtime files actually exist on disk
  ;    The WebView2 loader searches for the runtime in a well-known folder.
  IfFileExists "$LOCALAPPDATA\Microsoft\EdgeWebView\*.*" wv2_ok 0
  ; Also check the per-machine location
  IfFileExists "$PROGRAMFILES\Microsoft\EdgeWebView\*.*" wv2_ok 0
  ; Registry says installed but files are missing – fall through to install

wv2_not_found:
  ; Extract the embedded bootstrapper and run it
  IfFileExists "$INSTDIR\MicrosoftEdgeWebview2Setup.exe" 0 wv2_extract
  Delete "$INSTDIR\MicrosoftEdgeWebview2Setup.exe"

wv2_extract:
  SetOutPath "$INSTDIR"
  File "/oname=$INSTDIR\MicrosoftEdgeWebview2Setup.exe" "${WEBVIEW2BOOTSTRAPPERPATH}"
  DetailPrint "正在安装 WebView2 Runtime..."
  ExecWait '"$INSTDIR\MicrosoftEdgeWebview2Setup.exe" /install' $1
  Delete "$INSTDIR\MicrosoftEdgeWebview2Setup.exe"
  ${If} $1 <> 0
    MessageBox MB_ICONSTOP|MB_OK "WebView2 Runtime 安装失败（错误码 $1）。$\r$\n程序运行需要 WebView2，请手动安装：$\r$\nhttps://go.microsoft.com/fwlink/p/?LinkId=2124703"
    Abort
  ${EndIf}

wv2_ok:
  Pop $1
  Pop $0
!macroend
