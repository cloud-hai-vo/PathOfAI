/**
 * Portable Storage — all data stored relative to exe location
 * User can choose folder or defaults to exe directory
 * No AppData, no registry, fully portable on USB
 */

const fs = require("fs");
const path = require("path");

class PortableStorage {
  constructor(options = {}) {
    // Determine root: user-chosen folder OR exe directory
    this.root = options.customPath || this._detectExeDir();
    this.dataDir = path.join(this.root, "PathOfAI_Data");
    
    // Initialize directory structure
    this._ensureStructure();
  }

  _detectExeDir() {
    // In Tauri: process.env.APPIMAGE or path.dirname(process.execPath)
    // Fallback to current working directory
    if (process.env.PORTABLE_PATH) return process.env.PORTABLE_PATH;
    if (process.pkg) return path.dirname(process.execPath); // pkg bundled
    return process.cwd();
  }

  _ensureStructure() {
    const dirs = [
      this.dataDir,
      this.paths.config,
      this.paths.cache,
      this.paths.cacheImages,
      this.paths.cacheImagesItems,
      this.paths.cacheImagesGems,
      this.paths.cacheImagesFlasks,
      this.paths.cacheImagesCurrency,
      this.paths.cacheImagesSkills,
      this.paths.cachePrices,
      this.paths.backups,
      this.paths.snapshots,
      this.paths.knowledge,
      this.paths.knowledgeItems,
      this.paths.knowledgeGems,
      this.paths.knowledgeTree,
      this.paths.knowledgeCrafting,
      this.paths.knowledgeBuilds,
      this.paths.model,
      this.paths.logs,
    ];

    for (const dir of dirs) {
      if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }
    }
  }

  /** All paths relative to data directory */
  get paths() {
    const d = this.dataDir;
    return {
      // Config
      config: path.join(d, "config"),
      settings: path.join(d, "config", "settings.json"),
      aiProviders: path.join(d, "config", "ai-providers.json"),
      keybinds: path.join(d, "config", "keybinds.json"),
      alerts: path.join(d, "config", "alerts.json"),

      // Cache
      cache: path.join(d, "cache"),
      cacheImages: path.join(d, "cache", "images"),
      cacheImagesItems: path.join(d, "cache", "images", "items"),
      cacheImagesGems: path.join(d, "cache", "images", "gems"),
      cacheImagesFlasks: path.join(d, "cache", "images", "flasks"),
      cacheImagesCurrency: path.join(d, "cache", "images", "currency"),
      cacheImagesSkills: path.join(d, "cache", "images", "skills"),
      cachePrices: path.join(d, "cache", "prices"),
      cacheManifest: path.join(d, "cache", "manifest.json"),

      // Backups
      backups: path.join(d, "backups"),

      // Snapshots (build history)
      snapshots: path.join(d, "snapshots"),

      // Knowledge base (for AI RAG)
      knowledge: path.join(d, "knowledge"),
      knowledgeItems: path.join(d, "knowledge", "items"),
      knowledgeGems: path.join(d, "knowledge", "gems"),
      knowledgeTree: path.join(d, "knowledge", "tree"),
      knowledgeCrafting: path.join(d, "knowledge", "crafting"),
      knowledgeBuilds: path.join(d, "knowledge", "builds"),

      // Custom AI model
      model: path.join(d, "model"),
      modelWeights: path.join(d, "model", "seer.bin"),
      modelConfig: path.join(d, "model", "config.json"),
      modelVocab: path.join(d, "model", "vocab.json"),
      modelEmbeddings: path.join(d, "model", "embeddings.bin"),

      // Logs
      logs: path.join(d, "logs"),
    };
  }

  /** Folder structure on disk:
   *
   * PathOfAI.exe                     ← the app
   * PathOfAI_Data/                   ← all data here
   *   config/
   *     settings.json
   *     ai-providers.json
   *     keybinds.json
   *     alerts.json
   *   cache/
   *     images/
   *       items/                     ← real game art cached
   *       gems/
   *       flasks/
   *       currency/
   *       skills/
   *     prices/                      ← poe.ninja price cache
   *     manifest.json
   *   backups/                       ← PoB file backups
   *     MyBuild_2026-04-02_14-30.xml
   *   snapshots/                     ← build history snapshots
   *     build-name/
   *       snapshot-001.json
   *   knowledge/                     ← AI knowledge base
   *     items/
   *       unique_items.json
   *       base_types.json
   *       mod_tiers.json
   *     gems/
   *       active_gems.json
   *       support_gems.json
   *     tree/
   *       passive_nodes.json
   *       keystones.json
   *     crafting/
   *       bench_crafts.json
   *       fossil_mods.json
   *     builds/
   *       popular_builds.json
   *   model/                         ← custom Seer AI model
   *     seer.bin                     ← model weights (~50-100MB)
   *     config.json                  ← model architecture
   *     vocab.json                   ← tokenizer vocabulary
   *     embeddings.bin               ← knowledge embeddings
   *   logs/
   *     app.log
   */

  // =============================================
  // SETTINGS
  // =============================================

  loadSettings() {
    try {
      if (fs.existsSync(this.paths.settings)) {
        return JSON.parse(fs.readFileSync(this.paths.settings, "utf-8"));
      }
    } catch (e) { /* ignore */ }
    return this._defaultSettings();
  }

  saveSettings(settings) {
    fs.writeFileSync(this.paths.settings, JSON.stringify(settings, null, 2), "utf-8");
  }

  _defaultSettings() {
    return {
      pobPath: null,        // auto-detected or user-chosen
      theme: "blood",       // blood, dark, light
      language: "en",
      autoSync: true,
      syncInterval: 2000,   // ms
      aiProvider: "seer",   // seer (local), claude, gpt, etc
      overlayEnabled: false,
      overlayOpacity: 0.85,
      notifications: true,
      fontSize: "medium",
      colorBlindMode: false,
      maxBackups: 50,
      priceRefreshMinutes: 5,
    };
  }

  // =============================================
  // MIGRATION — move data to new location
  // =============================================

  async migrateToFolder(newPath) {
    const oldDataDir = this.dataDir;
    const newDataDir = path.join(newPath, "PathOfAI_Data");

    if (oldDataDir === newDataDir) return { success: true, message: "Same location" };

    try {
      // Copy all files
      this._copyDirRecursive(oldDataDir, newDataDir);
      
      // Update root
      this.root = newPath;
      this.dataDir = newDataDir;
      this._ensureStructure();

      return { success: true, message: `Migrated to ${newPath}` };
    } catch (err) {
      return { success: false, message: err.message };
    }
  }

  _copyDirRecursive(src, dest) {
    if (!fs.existsSync(dest)) fs.mkdirSync(dest, { recursive: true });
    const entries = fs.readdirSync(src, { withFileTypes: true });
    for (const entry of entries) {
      const srcPath = path.join(src, entry.name);
      const destPath = path.join(dest, entry.name);
      if (entry.isDirectory()) {
        this._copyDirRecursive(srcPath, destPath);
      } else {
        fs.copyFileSync(srcPath, destPath);
      }
    }
  }

  /** Get total data size */
  getDataSize() {
    let total = 0;
    const walk = (dir) => {
      if (!fs.existsSync(dir)) return;
      for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const p = path.join(dir, entry.name);
        if (entry.isDirectory()) walk(p);
        else total += fs.statSync(p).size;
      }
    };
    walk(this.dataDir);
    return { bytes: total, mb: Math.round(total / 1024 / 1024 * 10) / 10 };
  }
}

module.exports = PortableStorage;
