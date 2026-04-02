/**
 * PoB XML Parser — reads Path of Building XML into structured data
 * Works with both PoB Community Fork and original PoB
 */

class PoBParser {
  constructor(xmlString) {
    const parser = new DOMParser();
    this.doc = parser.parseFromString(xmlString, "text/xml");
    this.raw = xmlString;
  }

  /** Parse entire build into structured object */
  parse() {
    return {
      build: this.parseBuild(),
      tree: this.parseTree(),
      items: this.parseItems(),
      itemSets: this.parseItemSets(),
      skills: this.parseSkills(),
      config: this.parseConfig(),
      analysis: null, // filled by analyzer
    };
  }

  /** Parse <Build> section — class, level, stats */
  parseBuild() {
    const build = this.doc.querySelector("Build");
    if (!build) return null;

    const stats = {};
    build.querySelectorAll("PlayerStat").forEach((s) => {
      stats[s.getAttribute("stat")] = parseFloat(s.getAttribute("value"));
    });

    return {
      level: parseInt(build.getAttribute("level")) || 1,
      className: build.getAttribute("className") || "",
      ascendClassName: build.getAttribute("ascendClassName") || "",
      mainSocketGroup: parseInt(build.getAttribute("mainSocketGroup")) || 1,
      targetVersion: build.getAttribute("targetVersion") || "",
      pantheonMajor: build.getAttribute("pantheonMajorGod") || "",
      pantheonMinor: build.getAttribute("pantheonMinorGod") || "",
      bandit: build.getAttribute("bandit") || "None",
      stats,
    };
  }

  /** Parse <Tree> section — passive tree nodes */
  parseTree() {
    const tree = this.doc.querySelector("Tree");
    if (!tree) return null;

    const activeSpec = parseInt(tree.getAttribute("activeSpec")) || 1;
    const specs = [];

    tree.querySelectorAll("Spec").forEach((spec) => {
      const nodesStr = spec.getAttribute("nodes") || "";
      const nodes = nodesStr
        .split(",")
        .map((n) => n.trim())
        .filter(Boolean);
      const url = spec.querySelector("URL")?.textContent?.trim() || "";

      specs.push({
        treeVersion: spec.getAttribute("treeVersion") || "",
        ascendClassId: parseInt(spec.getAttribute("ascendClassId")) || 0,
        classId: parseInt(spec.getAttribute("classId")) || 0,
        nodes,
        url,
        nodeCount: nodes.length,
      });
    });

    return { activeSpec, specs };
  }

  /** Parse <Items> section — all items with mod extraction */
  parseItems() {
    const itemsEl = this.doc.querySelector("Items");
    if (!itemsEl) return [];

    const items = [];
    itemsEl.querySelectorAll("Item").forEach((item) => {
      items.push(this.parseItem(item));
    });
    return items;
  }

  /** Parse a single item element into structured data */
  parseItem(itemEl) {
    const id = parseInt(itemEl.getAttribute("id"));
    const text = itemEl.textContent.trim();
    const lines = text.split("\n").map((l) => l.trim()).filter(Boolean);

    const item = {
      id,
      rarity: "",
      name: "",
      base: "",
      quality: 0,
      sockets: "",
      levelReq: 0,
      implicits: [],
      explicits: [],
      tags: [],
      rawText: text,
      mods: [],
    };

    let implicitCount = 0;
    let pastImplicits = false;
    let implicitsSeen = 0;

    for (const line of lines) {
      if (line.startsWith("Rarity:")) {
        item.rarity = line.replace("Rarity:", "").trim();
      } else if (line.startsWith("Quality:")) {
        item.quality = parseInt(line.replace("Quality:", "").trim());
      } else if (line.startsWith("Sockets:")) {
        item.sockets = line.replace("Sockets:", "").trim();
      } else if (line.startsWith("LevelReq:")) {
        item.levelReq = parseInt(line.replace("LevelReq:", "").trim());
      } else if (line.startsWith("Implicits:")) {
        implicitCount = parseInt(line.replace("Implicits:", "").trim());
      } else if (line.startsWith("{tags:")) {
        item.tags = line.replace("{tags:", "").replace("}", "").split(",").map(t => t.trim());
      } else if (item.rarity && !item.name && !line.startsWith("{") && !line.includes(":")) {
        if (!item.name) item.name = line;
        else if (!item.base) item.base = line;
      } else if (line.match(/^[+\-\d%]/) || line.match(/^\d+% increased/) || line.match(/^Regenerate/) || line.match(/^Recover/) || line.match(/^Nearby/)) {
        const mod = this.parseMod(line);
        item.mods.push(mod);

        if (implicitsSeen < implicitCount) {
          item.implicits.push(mod);
          implicitsSeen++;
        } else {
          item.explicits.push(mod);
        }
      }
    }

    // Determine slot from tags
    if (item.tags.length > 0) {
      item.slot = item.tags[0];
    }

    return item;
  }

  /** Parse a mod line into structured data */
  parseMod(line) {
    const mod = { raw: line, stats: [] };

    // Match patterns like "+94 to maximum Life"
    const flatMatch = line.match(/([+\-]?\d+)\s+to\s+(maximum\s+)?(.+)/i);
    if (flatMatch) {
      mod.stats.push({
        type: "flat",
        value: parseInt(flatMatch[1]),
        stat: flatMatch[3].trim(),
      });
      return mod;
    }

    // Match patterns like "+42% to Fire Resistance"
    const resMatch = line.match(/([+\-]?\d+)%\s+to\s+(.+)/i);
    if (resMatch) {
      mod.stats.push({
        type: "percent",
        value: parseInt(resMatch[1]),
        stat: resMatch[2].trim(),
      });
      return mod;
    }

    // Match patterns like "15% increased Armour"
    const incMatch = line.match(/(\d+)%\s+(increased|reduced)\s+(.+)/i);
    if (incMatch) {
      const sign = incMatch[2] === "reduced" ? -1 : 1;
      mod.stats.push({
        type: "increased",
        value: parseInt(incMatch[1]) * sign,
        stat: incMatch[3].trim(),
      });
      return mod;
    }

    // Match "+1 to Level of all Fire Skill Gems"
    const gemLevelMatch = line.match(/([+\-]?\d+)\s+to\s+Level\s+of\s+(.+)/i);
    if (gemLevelMatch) {
      mod.stats.push({
        type: "gem_level",
        value: parseInt(gemLevelMatch[1]),
        stat: gemLevelMatch[2].trim(),
      });
      return mod;
    }

    // Catch-all
    mod.stats.push({ type: "unknown", raw: line });
    return mod;
  }

  /** Parse <ItemSet> mappings — which item goes in which slot */
  parseItemSets() {
    const itemsEl = this.doc.querySelector("Items");
    if (!itemsEl) return [];

    const sets = [];
    itemsEl.querySelectorAll("ItemSet").forEach((setEl) => {
      const slots = {};
      setEl.querySelectorAll("Slot").forEach((slot) => {
        slots[slot.getAttribute("name")] = parseInt(slot.getAttribute("itemId"));
      });
      sets.push({
        id: parseInt(setEl.getAttribute("id")),
        title: setEl.getAttribute("title") || "",
        slots,
      });
    });
    return sets;
  }

  /** Parse <Skills> section — all skill setups */
  parseSkills() {
    const skillsEl = this.doc.querySelector("Skills");
    if (!skillsEl) return [];

    const skillSets = [];
    skillsEl.querySelectorAll("SkillSet").forEach((setEl) => {
      const skills = [];
      setEl.querySelectorAll("Skill").forEach((skillEl) => {
        const gems = [];
        skillEl.querySelectorAll("Gem").forEach((gem) => {
          gems.push({
            gemId: gem.getAttribute("gemId") || "",
            level: parseInt(gem.getAttribute("level")) || 1,
            quality: parseInt(gem.getAttribute("quality")) || 0,
            enabled: gem.getAttribute("enabled") === "true",
            skillId: gem.getAttribute("skillId") || "",
          });
        });
        skills.push({
          label: skillEl.getAttribute("label") || "",
          enabled: skillEl.getAttribute("enabled") === "true",
          slot: skillEl.getAttribute("slot") || "",
          mainActiveSkill: parseInt(skillEl.getAttribute("mainActiveSkill")) || 0,
          gems,
        });
      });
      skillSets.push({
        id: parseInt(setEl.getAttribute("id")),
        skills,
      });
    });
    return skillSets;
  }

  /** Parse <Config> section — build configuration */
  parseConfig() {
    const configEl = this.doc.querySelector("Config");
    if (!configEl) return {};

    const config = {};
    configEl.querySelectorAll("Input").forEach((input) => {
      const name = input.getAttribute("name");
      if (input.hasAttribute("boolean")) {
        config[name] = input.getAttribute("boolean") === "true";
      } else if (input.hasAttribute("number")) {
        config[name] = parseFloat(input.getAttribute("number"));
      } else if (input.hasAttribute("string")) {
        config[name] = input.getAttribute("string");
      }
    });
    return config;
  }
}

export default PoBParser;
