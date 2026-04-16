// Embeds the Windows application icon and manifest into the final executable.
// No-op on non-Windows platforms.

fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/app.ico");
        res.set("FileDescription", "SilentWitness Screen Recorder & Keylogger");
        res.set("ProductName", "SilentWitness");
        res.set("CompanyName", "GoblinRules");
        res.set("LegalCopyright", "Non-commercial use only");
        // Request Per-Monitor-V2 DPI awareness and a modern common-controls manifest
        res.set_manifest(
            r#"
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <!-- Windows 10 / 11 -->
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
  </compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
    </windowsSettings>
  </application>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"/>
    </dependentAssembly>
  </dependency>
</assembly>
"#,
        );
        if let Err(e) = res.compile() {
            eprintln!("warning: failed to embed Windows resource: {e}");
        }
    }
}
