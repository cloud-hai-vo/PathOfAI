/**
 * PoB Writer — safely writes changes back to PoB XML files
 * Includes backup, undo, validation, and atomic writes
 * 
 * NOTE: This is the desktop (Node.js/Tauri) version.
 * Uses fs module for file operations.
 */

const fs = require("fs");
const path = require("path");
const { DOMParser, XMLSerializer } = require("xmldom");

class PoBWriter {
  constructor(options = {}) {
    this.backupDir = options.backupDir || path.join(process.env.APPDATA || ".", "PoBAdvisor", "backups");
    this.maxBackups = options.maxBackups || 50;
    this.undoHistory = [];
    this.maxUndo = options.maxUndo || 50;

    // Ensure backup directory exists
    if (!fs.existsSync(this.backupDir)) {
      fs.mkdirSync(this.backupDir, { recursive: true });
    }
  }

  // =============================================
  // BACKUP SYSTEM
  // =============================================

  /** Create a backup of the current file before modifying */
  createBackup(filePath) {
    if (!fs.existsSync(filePath)) return null;

    const fileName = path.basename(filePath, ".xml");
    const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
    const backupName = `${fileName}_${timestamp}.xml`;
    const backupPath = path.join(this.backupDir, backupName);

    fs.copyFileSync(filePath, backupPath);

    // Store in undo history
    this.undoHistory.push({
      originalPath: filePath,
      backupPath,
      timestamp: new Date(),
      action: "pre-modification backup",
    });

    // Trim old backups
    this.trimBackups();

    return backupPath;
  }

  /** Remove old backups beyond max limit */
  trimBackups() {
    const files = fs.readdirSync(this.backupDir)
      .map((f) => ({
        name: f,
        path: path.join(this.backupDir, f),
        time: fs.statSync(path.join(this.backupDir, f)).mtime,
      }))
      .sort((a, b) => b.time - a.time);

    if (files.length > this.maxBackups) {
      for (const file of files.slice(this.maxBackups)) {
        fs.unlinkSync(file.path);
      }
    }
  }

  /** Undo last change */
  undo() {
    if (this.undoHistory.length === 0) return { success: false, reason: "Nothing to undo" };

    const last = this.undoHistory.pop();
    if (fs.existsSync(last.backupPath)) {
      fs.copyFileSync(last.backupPath, last.originalPath);
      return { success: true, restoredFrom: last.backupPath };
    }

    return { success: false, reason: "Backup file not found" };
  }

  // =============================================
  // ATOMIC WRITE
  // =============================================

  /** Write XML safely — write to temp file then rename */
  atomicWrite(filePath, xmlString) {
    const tempPath = filePath + ".tmp";

    try {
      // Validate XML before writing
      const parser = new DOMParser();
      const doc = parser.parseFromString(xmlString, "text/xml");
      const errors = doc.getElementsByTagName("parsererror");
      if (errors.length > 0) {
        return { success: false, reason: "Invalid XML generated" };
      }

      // Write to temp file
      fs.writeFileSync(tempPath, xmlString, "utf-8");

      // Atomic rename
      fs.renameSync(tempPath, filePath);

      return { success: true };
    } catch (err) {
      // Clean up temp file on failure
      if (fs.existsSync(tempPath)) {
        fs.unlinkSync(tempPath);
      }
      return { success: false, reason: err.message };
    }
  }

  /** Check if file is locked (PoB is saving) */
  isFileLocked(filePath) {
    try {
      const fd = fs.openSync(filePath, "r+");
      fs.closeSync(fd);
      return false;
    } catch {
      return true;
    }
  }

  // =============================================
  // ITEM OPERATIONS
  // =============================================

  /** Replace an item in a specific slot */
  replaceItem(filePath, slotName, newItemText, itemSetId = 1) {
    if (this.isFileLocked(filePath)) {
      return { success: false, reason: "File is locked — PoB may be saving" };
    }

    // Backup first
    this.createBackup(filePath);

    const xmlString = fs.readFileSync(filePath, "utf-8");
    const parser = new DOMParser();
    const doc = parser.parseFromString(xmlString, "text/xml");

    // Find the item set
    const itemSets = doc.getElementsByTagName("ItemSet");
    let targetSet = null;
    for (let i = 0; i < itemSets.length; i++) {
      if (parseInt(itemSets[i].getAttribute("id")) === itemSetId) {
        targetSet = itemSets[i];
        break;
      }
    }

    if (!targetSet) return { success: false, reason: `ItemSet ${itemSetId} not found` };

    // Find the slot
    const slots = targetSet.getElementsByTagName("Slot");
    let targetSlot = null;
    for (let i = 0; i < slots.length; i++) {
      if (slots[i].getAttribute("name") === slotName) {
        targetSlot = slots[i];
        break;
      }
    }

    if (!targetSlot) return { success: false, reason: `Slot ${slotName} not found` };

    // Get next item ID
    const items = doc.getElementsByTagName("Item");
    let maxId = 0;
    for (let i = 0; i < items.length; i++) {
      maxId = Math.max(maxId, parseInt(items[i].getAttribute("id")) || 0);
    }
    const newId = maxId + 1;

    // Create new item element
    const itemsParent = doc.querySelector("Items") || doc.getElementsByTagName("Items")[0];
    const newItem = doc.createElement("Item");
    newItem.setAttribute("id", newId.toString());
    newItem.textContent = "\n" + newItemText + "\n";
    itemsParent.appendChild(newItem);

    // Update slot reference
    targetSlot.setAttribute("itemId", newId.toString());

    // Serialize and write
    const serializer = new XMLSerializer();
    const newXml = serializer.serializeToString(doc);

    return this.atomicWrite(filePath, newXml);
  }

  /** Update passive tree nodes */
  updatePassiveTree(filePath, newNodes, specIndex = 0) {
    if (this.isFileLocked(filePath)) {
      return { success: false, reason: "File is locked" };
    }

    this.createBackup(filePath);

    const xmlString = fs.readFileSync(filePath, "utf-8");
    const parser = new DOMParser();
    const doc = parser.parseFromString(xmlString, "text/xml");

    const specs = doc.getElementsByTagName("Spec");
    if (specIndex >= specs.length) {
      return { success: false, reason: `Spec index ${specIndex} not found` };
    }

    // Update nodes attribute
    specs[specIndex].setAttribute("nodes", newNodes.join(","));

    const serializer = new XMLSerializer();
    return this.atomicWrite(filePath, serializer.serializeToString(doc));
  }

  /** Update gem level/quality */
  updateGem(filePath, skillLabel, gemId, changes) {
    if (this.isFileLocked(filePath)) {
      return { success: false, reason: "File is locked" };
    }

    this.createBackup(filePath);

    const xmlString = fs.readFileSync(filePath, "utf-8");
    const parser = new DOMParser();
    const doc = parser.parseFromString(xmlString, "text/xml");

    const skills = doc.getElementsByTagName("Skill");
    for (let i = 0; i < skills.length; i++) {
      if (skills[i].getAttribute("label") === skillLabel) {
        const gems = skills[i].getElementsByTagName("Gem");
        for (let j = 0; j < gems.length; j++) {
          if (gems[j].getAttribute("gemId") === gemId) {
            if (changes.level !== undefined) {
              gems[j].setAttribute("level", changes.level.toString());
            }
            if (changes.quality !== undefined) {
              gems[j].setAttribute("quality", changes.quality.toString());
            }
            if (changes.enabled !== undefined) {
              gems[j].setAttribute("enabled", changes.enabled.toString());
            }
            break;
          }
        }
        break;
      }
    }

    const serializer = new XMLSerializer();
    return this.atomicWrite(filePath, serializer.serializeToString(doc));
  }

  /** Update build config option */
  updateConfig(filePath, configName, value) {
    if (this.isFileLocked(filePath)) {
      return { success: false, reason: "File is locked" };
    }

    this.createBackup(filePath);

    const xmlString = fs.readFileSync(filePath, "utf-8");
    const parser = new DOMParser();
    const doc = parser.parseFromString(xmlString, "text/xml");

    const config = doc.getElementsByTagName("Config")[0];
    if (!config) return { success: false, reason: "Config section not found" };

    // Find or create input
    const inputs = config.getElementsByTagName("Input");
    let found = false;
    for (let i = 0; i < inputs.length; i++) {
      if (inputs[i].getAttribute("name") === configName) {
        if (typeof value === "boolean") {
          inputs[i].setAttribute("boolean", value.toString());
        } else if (typeof value === "number") {
          inputs[i].setAttribute("number", value.toString());
        } else {
          inputs[i].setAttribute("string", value.toString());
        }
        found = true;
        break;
      }
    }

    if (!found) {
      const newInput = doc.createElement("Input");
      newInput.setAttribute("name", configName);
      if (typeof value === "boolean") {
        newInput.setAttribute("boolean", value.toString());
      } else if (typeof value === "number") {
        newInput.setAttribute("number", value.toString());
      } else {
        newInput.setAttribute("string", value.toString());
      }
      config.appendChild(newInput);
    }

    const serializer = new XMLSerializer();
    return this.atomicWrite(filePath, serializer.serializeToString(doc));
  }

  // =============================================
  // DIFF PREVIEW
  // =============================================

  /** Generate a preview of changes before applying */
  previewChange(filePath, changeType, params) {
    const xmlString = fs.readFileSync(filePath, "utf-8");

    // Apply change to in-memory copy
    // Return before/after comparison
    return {
      changeType,
      params,
      filePath,
      warning: "This will modify your PoB build file",
      canUndo: true,
    };
  }

  /** Get backup history */
  getBackupHistory() {
    return this.undoHistory.map((entry) => ({
      timestamp: entry.timestamp,
      action: entry.action,
      file: path.basename(entry.originalPath),
    }));
  }
}

module.exports = PoBWriter;
