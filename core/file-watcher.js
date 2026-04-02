/**
 * PoB File Watcher — monitors build files for changes
 * Auto-syncs with PoB on every save
 * Designed for Tauri/Node.js desktop runtime
 */

const fs = require("fs");
const path = require("path");
const PoBParser = require("./pob-parser");
const BuildAnalyzer = require("./build-analyzer");

class PoBFileWatcher {
  constructor(options = {}) {
    this.watchPaths = [];
    this.watchers = [];
    this.builds = new Map(); // filepath -> parsed build data
    this.snapshots = new Map(); // filepath -> array of snapshots
    this.debounceTimers = new Map();
    this.debounceMs = options.debounceMs || 500;
    this.onChange = options.onChange || (() => {});
    this.onError = options.onError || console.error;
    this.maxSnapshots = options.maxSnapshots || 100;
    this.autoAnalyze = options.autoAnalyze !== false;
  }

  // =============================================
  // DISCOVERY — find PoB install
  // =============================================

  /** Auto-detect Path of Building install locations */
  static detectPoBPaths() {
    const appData = process.env.APPDATA || "";
    const candidates = [
      path.join(appData, "Path of Building", "Builds"),
      path.join(appData, "Path of Building Community", "Builds"),
      path.join(appData, "PathOfBuilding", "Builds"),
    ];

    const found = [];
    for (const dir of candidates) {
      if (fs.existsSync(dir)) {
        found.push(dir);
      }
    }

    return found;
  }

  /** Scan a directory for all .xml build files */
  static scanBuildFiles(directory) {
    const files = [];

    function walk(dir) {
      if (!fs.existsSync(dir)) return;
      const entries = fs.readdirSync(dir, { withFileTypes: true });
      for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory()) {
          walk(fullPath);
        } else if (entry.name.endsWith(".xml")) {
          const stat = fs.statSync(fullPath);
          files.push({
            path: fullPath,
            name: entry.name.replace(".xml", ""),
            directory: dir,
            size: stat.size,
            modified: stat.mtime,
          });
        }
      }
    }

    walk(directory);
    return files.sort((a, b) => b.modified - a.modified);
  }

  // =============================================
  // WATCHING — monitor files for changes
  // =============================================

  /** Start watching a directory for build file changes */
  watchDirectory(directory) {
    if (!fs.existsSync(directory)) {
      this.onError(`Directory not found: ${directory}`);
      return false;
    }

    this.watchPaths.push(directory);

    // Initial scan
    const files = PoBFileWatcher.scanBuildFiles(directory);
    for (const file of files) {
      this._loadBuild(file.path);
    }

    // Watch for changes
    const watcher = fs.watch(directory, { recursive: true }, (event, filename) => {
      if (!filename || !filename.endsWith(".xml")) return;
      const fullPath = path.join(directory, filename);
      this._handleFileChange(event, fullPath);
    });

    this.watchers.push(watcher);
    return true;
  }

  /** Watch a specific build file */
  watchFile(filePath) {
    if (!fs.existsSync(filePath)) {
      this.onError(`File not found: ${filePath}`);
      return false;
    }

    this._loadBuild(filePath);

    const watcher = fs.watch(filePath, (event) => {
      this._handleFileChange(event, filePath);
    });

    this.watchers.push(watcher);
    return true;
  }

  /** Stop all watchers */
  stopAll() {
    for (const watcher of this.watchers) {
      watcher.close();
    }
    this.watchers = [];
    this.debounceTimers.forEach(timer => clearTimeout(timer));
    this.debounceTimers.clear();
  }

  // =============================================
  // SNAPSHOTS — track build changes over time
  // =============================================

  /** Get snapshot history for a build file */
  getSnapshots(filePath) {
    return this.snapshots.get(filePath) || [];
  }

  /** Create a manual snapshot */
  createSnapshot(filePath, label = "") {
    const build = this.builds.get(filePath);
    if (!build) return null;

    const snapshot = {
      timestamp: new Date(),
      label: label || `Manual snapshot`,
      stats: { ...build.build?.stats },
      itemScores: build.items?.map(i => ({ slot: i.slot || i.tags?.[0], score: i.score })),
      dps: build.build?.stats?.TotalDPS || build.build?.stats?.FireDotDPS || 0,
      life: build.build?.stats?.Life || 0,
    };

    this._addSnapshot(filePath, snapshot);
    return snapshot;
  }

  /** Compare two snapshots */
  compareSnapshots(filePath, index1, index2) {
    const snaps = this.snapshots.get(filePath) || [];
    const s1 = snaps[index1];
    const s2 = snaps[index2];
    if (!s1 || !s2) return null;

    return {
      timeSpan: Math.abs(s2.timestamp - s1.timestamp),
      dpsChange: s2.dps - s1.dps,
      dpsPercent: s1.dps > 0 ? ((s2.dps / s1.dps) - 1) * 100 : 0,
      lifeChange: s2.life - s1.life,
      statChanges: this._diffStats(s1.stats, s2.stats),
    };
  }

  // =============================================
  // BUILD ACCESS
  // =============================================

  /** Get parsed build data for a file */
  getBuild(filePath) {
    return this.builds.get(filePath) || null;
  }

  /** Get all loaded builds */
  getAllBuilds() {
    const result = [];
    for (const [path, build] of this.builds) {
      result.push({ path, build });
    }
    return result;
  }

  /** Force reload a build file */
  reloadBuild(filePath) {
    return this._loadBuild(filePath);
  }

  // =============================================
  // INTERNAL
  // =============================================

  _handleFileChange(event, filePath) {
    // Debounce — PoB may write multiple times in quick succession
    const existing = this.debounceTimers.get(filePath);
    if (existing) clearTimeout(existing);

    this.debounceTimers.set(filePath, setTimeout(() => {
      this.debounceTimers.delete(filePath);

      if (event === "rename" && !fs.existsSync(filePath)) {
        // File deleted or renamed
        this.builds.delete(filePath);
        this.onChange({ type: "deleted", path: filePath });
        return;
      }

      const oldBuild = this.builds.get(filePath);
      const newBuild = this._loadBuild(filePath);

      if (newBuild) {
        // Auto-snapshot on change
        const changes = this._detectChanges(oldBuild, newBuild);
        if (changes.hasChanges) {
          const snapshot = {
            timestamp: new Date(),
            label: changes.description,
            stats: { ...newBuild.build?.stats },
            dps: newBuild.build?.stats?.TotalDPS || newBuild.build?.stats?.FireDotDPS || 0,
            life: newBuild.build?.stats?.Life || 0,
            dpsChange: changes.dpsChange,
            lifeChange: changes.lifeChange,
          };
          this._addSnapshot(filePath, snapshot);
        }

        this.onChange({
          type: "changed",
          path: filePath,
          build: newBuild,
          changes,
        });
      }
    }, this.debounceMs));
  }

  _loadBuild(filePath) {
    try {
      if (!fs.existsSync(filePath)) return null;

      const xml = fs.readFileSync(filePath, "utf-8");
      const parser = new PoBParser(xml);
      const buildData = parser.parse();

      if (this.autoAnalyze) {
        const analyzer = new BuildAnalyzer(buildData);
        buildData.analysis = analyzer.analyze();
      }

      this.builds.set(filePath, buildData);
      return buildData;
    } catch (err) {
      this.onError(`Failed to parse ${filePath}: ${err.message}`);
      return null;
    }
  }

  _detectChanges(oldBuild, newBuild) {
    if (!oldBuild) return { hasChanges: true, description: "Initial load", dpsChange: 0, lifeChange: 0 };

    const oldStats = oldBuild.build?.stats || {};
    const newStats = newBuild.build?.stats || {};

    const oldDps = oldStats.TotalDPS || oldStats.FireDotDPS || 0;
    const newDps = newStats.TotalDPS || newStats.FireDotDPS || 0;
    const oldLife = oldStats.Life || 0;
    const newLife = newStats.Life || 0;

    const dpsChange = newDps - oldDps;
    const lifeChange = newLife - oldLife;

    // Detect what changed
    const changes = [];
    if (Math.abs(dpsChange) > 100) changes.push(`DPS ${dpsChange > 0 ? "+" : ""}${Math.round(dpsChange)}`);
    if (Math.abs(lifeChange) > 10) changes.push(`Life ${lifeChange > 0 ? "+" : ""}${lifeChange}`);

    // Check item changes
    const oldItems = oldBuild.items || [];
    const newItems = newBuild.items || [];
    for (let i = 0; i < Math.max(oldItems.length, newItems.length); i++) {
      if (oldItems[i]?.rawText !== newItems[i]?.rawText) {
        changes.push(`Item changed: slot ${newItems[i]?.slot || i}`);
      }
    }

    // Check tree changes
    const oldNodes = oldBuild.tree?.specs?.[0]?.nodes || [];
    const newNodes = newBuild.tree?.specs?.[0]?.nodes || [];
    if (oldNodes.length !== newNodes.length || oldNodes.join(",") !== newNodes.join(",")) {
      const diff = newNodes.length - oldNodes.length;
      changes.push(`Tree: ${diff >= 0 ? "+" : ""}${diff} nodes`);
    }

    return {
      hasChanges: changes.length > 0,
      description: changes.join(", ") || "No changes detected",
      dpsChange,
      lifeChange,
      details: changes,
    };
  }

  _addSnapshot(filePath, snapshot) {
    if (!this.snapshots.has(filePath)) {
      this.snapshots.set(filePath, []);
    }
    const snaps = this.snapshots.get(filePath);
    snaps.unshift(snapshot);

    // Trim old snapshots
    if (snaps.length > this.maxSnapshots) {
      snaps.length = this.maxSnapshots;
    }
  }

  _diffStats(stats1, stats2) {
    const diff = {};
    const allKeys = new Set([...Object.keys(stats1 || {}), ...Object.keys(stats2 || {})]);
    for (const key of allKeys) {
      const v1 = stats1?.[key] || 0;
      const v2 = stats2?.[key] || 0;
      if (v1 !== v2) {
        diff[key] = { before: v1, after: v2, change: v2 - v1 };
      }
    }
    return diff;
  }
}

module.exports = PoBFileWatcher;
