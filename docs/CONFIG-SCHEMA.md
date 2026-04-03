# Path of AI — Configuration Schema

## settings.json

Stored in `PathOfAI_Data/settings.json`. Created on first launch by wizard.

```json
{
  "_version": "1.0",
  
  "paths": {
    "pob_directory": "C:/Users/.../Path of Building/",
    "data_directory": "./PathOfAI_Data/",
    "backup_directory": "./PathOfAI_Data/backups/"
  },
  
  "game": {
    "version": "poe1",
    "league": "Mirage",
    "mode": "softcore_trade"
  },
  
  "ai_provider": {
    "primary": "seer",
    "cloud_provider": null,
    "cloud_model": null,
    "auto_escalate": true,
    "escalation_threshold": 0.7
  },
  
  "calculation": {
    "enable_pob_verification": false,
    "fast_estimate_enabled": true,
    "conditional_buffs_default": "mapping"
  },
  
  "market": {
    "price_refresh_interval_ms": 300000,
    "price_cache_ttl_ms": 300000,
    "trade_league": "Mirage"
  },
  
  "file_watcher": {
    "enabled": true,
    "debounce_ms": 500,
    "watch_subfolders": true
  },
  
  "ui": {
    "theme": "dark",
    "font_size": "medium",
    "show_pob_verification": false,
    "default_panel": "prophecy",
    "animations_enabled": true,
    "color_blind_mode": false
  },
  
  "auto_update": {
    "check_on_startup": true,
    "check_interval_ms": 21600000,
    "auto_download": false
  },
  
  "privacy": {
    "send_build_data_to_cloud": false,
    "anonymize_account_name": true,
    "telemetry_opt_in": false
  },
  
  "renderer": {
    "tier": "auto",
    "options": ["auto", "canvas2d", "native_gpu"],
    "target_fps": 60,
    "particle_quality": "medium",
    "show_damage_numbers": true,
    "show_aura_rings": true,
    "combat_sim_speed": 1.0
  },
  
  "overlay": {
    "enabled": false,
    "opacity": 0.9,
    "position": "top-right",
    "scale": 1.0,
    "always_on_top": true,
    "show_in_taskbar": true
  },
  
  "notifications": {
    "enabled": true,
    "sound_enabled": true,
    "sound_volume": 0.7,
    "sound_file": "default",
    "popup_duration_ms": 5000,
    "price_alerts": true,
    "build_changes": true,
    "update_available": true
  },
  
  "hotkeys": {
    "paste_item": "Ctrl+Shift+V",
    "toggle_overlay": "Ctrl+Shift+P",
    "refresh_build": "F5",
    "open_grimoire": "Ctrl+Shift+G",
    "open_prophecy": "Ctrl+Shift+U",
    "open_forge": "Ctrl+Shift+F",
    "open_stash": "Ctrl+Shift+S",
    "undo": "Ctrl+Z",
    "redo": "Ctrl+Shift+Z"
  },
  
  "backup": {
    "max_snapshots_per_build": 50,
    "max_backup_age_days": 30
  }
}
```

## tauri.conf.json (Tauri-specific)

```json
{
  "productName": "Path of AI",
  "version": "0.1.0",
  "identifier": "com.pathofai.app",
  "build": {
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [{
      "title": "PATH of AI",
      "width": 1280,
      "height": 800,
      "minWidth": 900,
      "minHeight": 600,
      "resizable": true,
      "fullscreen": false
    }],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; connect-src 'self' https://poe.ninja https://api.anthropic.com https://api.openai.com"
    }
  },
  "plugins": {
    "stronghold": { "enabled": true },
    "fs": { "enabled": true, "scope": ["$APPDATA/**", "$HOME/**"] },
    "dialog": { "enabled": true },
    "notification": { "enabled": true },
    "global-shortcut": { "enabled": true },
    "updater": {
      "enabled": true,
      "endpoints": ["https://github.com/path-of-ai/releases/latest/download/update.json"]
    }
  }
}
```
