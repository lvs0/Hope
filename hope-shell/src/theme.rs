//! Deep Space theme for Hope Shell

use anyhow::Result;
use tracing::info;

use crate::config::ThemeConfig;

/// Apply the Deep Space theme system-wide
pub fn apply_deep_space(theme: &ThemeConfig) -> Result<()> {
    info!("Applying Deep Space theme: {}", theme.name);

    // Set GTK theme environment variables
    std::env::set_var("GTK_THEME", "Hope-DeepSpace");
    std::env::set_var("ICON_THEME", "Papirus-Dark");

    // Set cursor theme
    std::env::set_var("XCURSOR_THEME", "Adwaita");
    std::env::set_var("XCURSOR_SIZE", "24");

    // Create GTK4 settings
    apply_gtk4_settings(theme)?;

    // Create Waybar theme
    apply_waybar_theme(theme)?;

    // Create Foot terminal theme (Wayland-native terminal)
    apply_foot_theme(theme)?;

    Ok(())
}

/// Generate GTK4 settings for Deep Space
fn apply_gtk4_settings(_theme: &ThemeConfig) -> Result<()> {
    let gtk_config_dir = dirs::config_dir()
        .map(|d| d.join("gtk-4.0"))
        .unwrap_or_else(|| "/tmp/hope-gtk".into());

    std::fs::create_dir_all(&gtk_config_dir)?;

    let settings = format!(
        r#"# Hope OS — GTK4 Deep Space Theme
[Settings]
gtk-application-prefer-dark-theme=true
gtk-theme-name=Hope-DeepSpace
gtk-icon-theme-name=Papirus-Dark
gtk-cursor-theme-name=Adwaita
gtk-cursor-theme-size=24
gtk-font-name=Inter 10
gtk-decoration-layout=close,minimize,maximize
gtk-enable-animations=true
gtk-enable-primary-paste=true
gtk-recent-files-enabled=false
gtk-modules=appmenu-gtk-module
"#,
    );

    let settings_path = gtk_config_dir.join("settings.ini");
    std::fs::write(&settings_path, settings)?;
    info!("GTK4 settings written to {}", settings_path.display());

    Ok(())
}

/// Generate Waybar theme
fn apply_waybar_theme(theme: &ThemeConfig) -> Result<()> {
    let waybar_dir = dirs::config_dir()
        .map(|d| d.join("waybar"))
        .unwrap_or_else(|| "/tmp/hope-waybar".into());

    std::fs::create_dir_all(&waybar_dir)?;

    let css = format!(
        r#"/* Hope OS — Waybar Deep Space Theme */

* {{
    font-family: "Inter", "Symbols Nerd Font", sans-serif;
    font-size: 13px;
    color: {fg};
}}

window#waybar {{
    background-color: {bg};
    border-top: 1px solid {primary};
}}

#workspaces button {{
    padding: 0 8px;
    color: {fg};
    background-color: transparent;
    border-radius: 4px;
}}

#workspaces button.active {{
    background-color: {primary};
    color: #FFFFFF;
}}

#clock, #battery, #pulseaudio, #network, #tray {{
    padding: 0 10px;
    margin: 4px 2px;
    border-radius: 4px;
}}

#battery {{
    background-color: {success};
}}

#battery.warning {{
    background-color: {secondary};
}}

#battery.critical {{
    background-color: {error};
}}
"#,
        bg = theme.background,
        fg = theme.foreground,
        primary = theme.accents.primary,
        secondary = theme.accents.secondary,
        error = theme.accents.error,
        success = theme.accents.success,
    );

    let css_path = waybar_dir.join("style.css");
    std::fs::write(&css_path, css)?;
    info!("Waybar theme written to {}", css_path.display());

    Ok(())
}

/// Generate Foot terminal color scheme
fn apply_foot_theme(theme: &ThemeConfig) -> Result<()> {
    let foot_dir = dirs::config_dir()
        .map(|d| d.join("foot"))
        .unwrap_or_else(|| "/tmp/hope-foot".into());

    std::fs::create_dir_all(&foot_dir)?;

    let config = format!(
        r#"# Hope OS — Foot Terminal Deep Space Theme

[colors]
background={bg}
foreground={fg}

[colors.normal]
black={bg}
red={error}
green={success}
yellow=#F59E0B
blue={primary}
magenta={secondary}
cyan={tertiary}
white=#FFFFFF

[colors.bright]
black=#374151
red=#F87171
green=#4ADE80
yellow=#FCD34D
blue=#818CF8
magenta=#A78BFA
cyan=#22D3EE
white=#F9FAFB

[csd]
preferred=server
"#,
        bg = theme.background,
        fg = theme.foreground,
        primary = theme.accents.primary,
        secondary = theme.accents.secondary,
        tertiary = theme.accents.tertiary,
        error = theme.accents.error,
        success = theme.accents.success,
    );

    let config_path = foot_dir.join("foot.ini");
    std::fs::write(&config_path, config)?;
    info!("Foot theme written to {}", config_path.display());

    Ok(())
}

/// Get the Deep Space wallpaper path
pub fn wallpaper_path() -> Option<String> {
    let paths = [
        "/usr/share/hope/wallpapers/deep-space.png",
        "/usr/share/backgrounds/hope/deep-space.png",
        "/usr/share/wallpapers/hope/deep-space.png",
    ];

    for path in &paths {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_space_colors() {
        let theme = ThemeConfig {
            name: "deep-space".to_string(),
            background: "#0F0F12".to_string(),
            foreground: "#E0E0E8".to_string(),
            accents: crate::config::AccentColors {
                primary: "#6366F1".to_string(),
                secondary: "#8B5CF6".to_string(),
                tertiary: "#06B6D4".to_string(),
                error: "#EF4444".to_string(),
                success: "#22C55E".to_string(),
            },
            blur: 5,
            opacity: 0.95,
        };

        assert_eq!(theme.background, "#0F0F12");
        assert_eq!(theme.accents.primary, "#6366F1");
    }
}
