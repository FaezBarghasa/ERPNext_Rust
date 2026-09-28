//! Universal Single-Binary Distribution, Packaging (.deb, .exe, .dmg), and Dual-Mode Runtime.
//!
//! Provides:
//! - Dual-personality runtime: `HeadlessServer` (Axum/Actix + auto-TLS) vs `NativeDesktop` (Wry/Tao loopback).
//! - Cross-platform service daemonization: Linux systemd, Windows Service Control Manager, macOS LaunchDaemon.
//! - Automated distribution package scaffolding for Debian/Ubuntu `.deb`, Windows `.exe` (Inno Setup), and macOS Universal `.dmg`.

use serde::{Deserialize, Serialize};

/// Target operating system and packaging platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetPlatform {
    LinuxDebian,
    WindowsInno,
    MacOsDmg,
}

/// Dual-personality runtime mode for the unified binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeMode {
    /// Headless multi-user background server daemon with Tokio workers and auto-TLS.
    HeadlessServer,
    /// Local desktop single-user GUI embedding WebView loopback interface (<30MB RAM).
    NativeDesktop,
}

/// Generated deployment artifact metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageScaffold {
    pub platform: TargetPlatform,
    pub version: String,
    pub package_name: String,
    pub service_definition: String,
    pub desktop_launcher_entry: Option<String>,
    pub installer_script: String,
}

/// Packaging and deployment builder.
pub struct UniversalDistributionBuilder;

impl UniversalDistributionBuilder {
    /// Generates Debian/Ubuntu .deb package metadata, systemd service, and XDG desktop entry.
    pub fn scaffold_debian_package(version: &str, arch: &str) -> PackageScaffold {
        let systemd_service = r#"[Unit]
Description=ERPNext Enterprise Core Daemon
After=network.target network-online.target
Wants=network-online.target

[Service]
Type=simple
User=erpnext
Group=erpnext
ExecStart=/usr/bin/erpnext --server --config /etc/erpnext/config.toml
Restart=always
RestartSec=5s
LimitNOFILE=65536
Environment="RUST_LOG=info"
WorkingDirectory=/var/lib/erpnext

[Install]
WantedBy=multi-user.target
"#;

        let desktop_entry = r#"[Desktop Entry]
Name=ERPNext Enterprise Desk
Comment=Pure-Rust Modern Enterprise Operating System
Exec=/usr/bin/erpnext --desktop
Icon=erpnext
Terminal=false
Type=Application
Categories=Office;Finance;Accounting;
StartupNotify=true
"#;

        let postinst_script = r#"#!/bin/sh
set -e
# Create system user & directories
if ! id -u erpnext >/dev/null 2>&1; then
    useradd --system --shell /usr/sbin/nologin --home-dir /var/lib/erpnext erpnext
fi
mkdir -p /etc/erpnext /var/lib/erpnext/data /var/log/erpnext
chown -R erpnext:erpnext /var/lib/erpnext /var/log/erpnext
chmod 750 /var/lib/erpnext /var/log/erpnext

# Systemd daemon reload
if [ -d /run/systemd/system ]; then
    systemctl --system daemon-reload >/dev/null || true
fi
exit 0
"#
        .to_string();

        let installer_script = format!(
            r#"Package: erpnext
Version: {version}
Architecture: {arch}
Maintainer: Faez Barghasa <faez.barghasa.org@gmail.com>
Installed-Size: 42000
Depends: libc6 (>= 2.34), libssl3
Section: office
Priority: optional
Description: High-performance, memory-safe, zero-cost abstraction enterprise suite in Rust.
"#
        );

        PackageScaffold {
            platform: TargetPlatform::LinuxDebian,
            version: version.to_string(),
            package_name: format!("erpnext_{version}_{arch}.deb"),
            service_definition: systemd_service.to_string(),
            desktop_launcher_entry: Some(desktop_entry.to_string()),
            installer_script: format!("{installer_script}\n---\nPOSTINST:\n{postinst_script}"),
        }
    }

    /// Generates Windows Inno Setup installer script and native Windows Service configuration.
    pub fn scaffold_windows_package(version: &str) -> PackageScaffold {
        let windows_service_spec = r#"<service>
  <id>ERPNextService</id>
  <name>ERPNext Enterprise Engine</name>
  <description>High-Performance Pure-Rust Enterprise System Daemon</description>
  <executable>%BASE%\erpnext.exe</executable>
  <arguments>--server --service</arguments>
  <logmode>rotate</logmode>
  <startmode>Automatic</startmode>
</service>
"#;

        let inno_script = format!(
            r#"[Setup]
AppName=ERPNext Enterprise Suite
AppVersion={version}
DefaultDirName={{autopf}}\ERPNext
DefaultGroupName=ERPNext
OutputDir=dist
OutputBaseFilename=ERPNext_Setup_{version}
Compression=lzma2/ultra64
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop icon"; GroupDescription: "Additional icons:"; Flags: unchecked
Name: "winservice"; Description: "Install and start as Windows Service (Server Mode)"; GroupDescription: "Deployment options:"

[Files]
Source: "target\release\erpnext.exe"; DestDir: "{{app}}"; Flags: ignoreversion
Source: "assets\*"; DestDir: "{{app}}\assets"; Flags: ignoreversion recursesubdirs

[Icons]
Name: "{{autoprograms}}\ERPNext"; Filename: "{{app}}\erpnext.exe"; Parameters: "--desktop"
Name: "{{autodesktop}}\ERPNext"; Filename: "{{app}}\erpnext.exe"; Parameters: "--desktop"; Tasks: desktopicon

[Run]
Filename: "{{app}}\erpnext.exe"; Parameters: "service install"; Tasks: winservice; Flags: runhidden
Filename: "{{app}}\erpnext.exe"; Parameters: "--desktop"; Description: "Launch ERPNext Desk"; Flags: postinstall nowait skipifsilent
"#
        );

        PackageScaffold {
            platform: TargetPlatform::WindowsInno,
            version: version.to_string(),
            package_name: format!("ERPNext_Setup_{version}.exe"),
            service_definition: windows_service_spec.to_string(),
            desktop_launcher_entry: None,
            installer_script: inno_script,
        }
    }

    /// Generates macOS LaunchDaemon plist and universal DMG packaging specifications.
    pub fn scaffold_macos_package(version: &str) -> PackageScaffold {
        let launchd_plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.erpnext.server</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Applications/ERPNext.app/Contents/MacOS/erpnext</string>
        <string>--server</string>
        <string>--config</string>
        <string>/Library/Application Support/ERPNext/config.toml</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/var/log/erpnext.stdout.log</string>
    <key>StandardErrorPath</key>
    <string>/var/log/erpnext.stderr.log</string>
</dict>
</plist>
"#;

        let dmg_builder_script = format!(
            r#"#!/bin/bash
set -e
echo "Building Universal macOS Binary & DMG for ERPNext v{version}..."

# 1. Build x86_64 & aarch64 universal binary
lipo -create -output ERPNext.app/Contents/MacOS/erpnext \
    target/x86_64-apple-darwin/release/erpnext \
    target/aarch64-apple-darwin/release/erpnext

# 2. Codesign application bundle with hardened runtime
codesign --force --options runtime --deep --sign "Developer ID Application: Faez Barghasa" ERPNext.app

# 3. Create drag-and-drop DMG
create-dmg \
  --volname "ERPNext Installer" \
  --window-pos 200 120 \
  --window-size 800 400 \
  --icon-size 100 \
  --icon "ERPNext.app" 200 190 \
  --hide-extension "ERPNext.app" \
  --app-drop-link 600 185 \
  "dist/ERPNext_{version}_Universal.dmg" \
  "ERPNext.app"

echo "✅ DMG generated: dist/ERPNext_{version}_Universal.dmg"
"#
        );

        PackageScaffold {
            platform: TargetPlatform::MacOsDmg,
            version: version.to_string(),
            package_name: format!("ERPNext_{version}_Universal.dmg"),
            service_definition: launchd_plist.to_string(),
            desktop_launcher_entry: Some("ERPNext.app/Contents/Info.plist".into()),
            installer_script: dmg_builder_script,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debian_package_scaffolding() {
        let pkg = UniversalDistributionBuilder::scaffold_debian_package("0.2.0", "amd64");
        assert_eq!(pkg.package_name, "erpnext_0.2.0_amd64.deb");
        assert!(
            pkg.service_definition
                .contains("ExecStart=/usr/bin/erpnext --server")
        );
        assert!(
            pkg.desktop_launcher_entry
                .unwrap()
                .contains("Categories=Office;Finance;Accounting;")
        );
    }

    #[test]
    fn test_windows_package_scaffolding() {
        let pkg = UniversalDistributionBuilder::scaffold_windows_package("0.2.0");
        assert_eq!(pkg.package_name, "ERPNext_Setup_0.2.0.exe");
        assert!(pkg.service_definition.contains("<id>ERPNextService</id>"));
        assert!(
            pkg.installer_script
                .contains("ArchitecturesInstallIn64BitMode=x64")
        );
    }

    #[test]
    fn test_macos_package_scaffolding() {
        let pkg = UniversalDistributionBuilder::scaffold_macos_package("0.2.0");
        assert_eq!(pkg.package_name, "ERPNext_0.2.0_Universal.dmg");
        assert!(
            pkg.service_definition
                .contains("<string>com.erpnext.server</string>")
        );
        assert!(pkg.installer_script.contains("lipo -create"));
    }
}
