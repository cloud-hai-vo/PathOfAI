/**
 * Build Analyzer — scores items, detects issues, generates suggestions
 * Pure local logic, no API calls needed
 */

class BuildAnalyzer {
  constructor(buildData) {
    this.build = buildData.build;
    this.items = buildData.items;
    this.itemSets = buildData.itemSets;
    this.skills = buildData.skills;
    this.config = buildData.config;
    this.tree = buildData.tree;
  }

  /** Run full analysis */
  analyze() {
    return {
      overallScore: this.calculateOverallScore(),
      defenses: this.analyzeDefenses(),
      offense: this.analyzeOffense(),
      items: this.analyzeItems(),
      gems: this.analyzeGems(),
      issues: this.detectIssues(),
      suggestions: this.generateSuggestions(),
      progression: this.assessProgression(),
      checklist: this.generateChecklist(),
    };
  }

  // =============================================
  // DEFENSE ANALYSIS
  // =============================================

  analyzeDefenses() {
    const stats = this.build?.stats || {};

    const life = stats.Life || 0;
    const es = stats.EnergyShield || 0;
    const armour = stats.Armour || 0;
    const evasion = stats.Evasion || 0;
    const block = stats.BlockChance || 0;
    const spellBlock = stats.SpellBlockChance || 0;

    const resists = {
      fire: stats.FireResist || 0,
      cold: stats.ColdResist || 0,
      lightning: stats.LightningResist || 0,
      chaos: stats.ChaosResist || 0,
    };

    const maxRes = {
      fire: 75 + (stats.MaxFireResist || 0),
      cold: 75,
      lightning: 75,
    };

    // Overcap for curse maps (-24% from Elemental Weakness)
    const overcap = {
      fire: resists.fire - maxRes.fire,
      cold: resists.cold - 75,
      lightning: resists.lightning - 75,
    };

    // Physical damage reduction from armour (vs 5000 hit)
    const physReduction = this.calculatePhysReduction(armour, 5000);

    // Effective HP calculation
    const ehpPhys = life / (1 - physReduction / 100);
    const ehpEle = life / (1 - 0.75); // at cap

    return {
      life,
      energyShield,
      totalPool: life + es,
      armour,
      evasion,
      block,
      spellBlock,
      resists,
      maxResists: maxRes,
      overcap,
      chaosResistTier: this.chaosResTier(resists.chaos),
      physReduction,
      ehpPhysical: Math.round(ehpPhys),
      ehpElemental: Math.round(ehpEle),
      lifeRegen: stats.LifeRegen || 0,
      ailmentImmunity: this.checkAilmentImmunity(),
      defenseLayers: this.countDefenseLayers(),
      score: this.scoreDefenses(life, resists, armour, block),
    };
  }

  calculatePhysReduction(armour, damage) {
    // PoE formula: Armour / (Armour + 5 * Damage)
    if (armour <= 0) return 0;
    return Math.min(90, (armour / (armour + 5 * damage)) * 100);
  }

  chaosResTier(chaosRes) {
    if (chaosRes >= 75) return { tier: "capped", label: "Excellent", color: "green" };
    if (chaosRes >= 50) return { tier: "good", label: "Good", color: "blue" };
    if (chaosRes >= 0) return { tier: "okay", label: "Okay", color: "yellow" };
    if (chaosRes >= -30) return { tier: "low", label: "Dangerous", color: "orange" };
    return { tier: "negative", label: "Critical!", color: "red" };
  }

  checkAilmentImmunity() {
    // Basic detection from build data — full version would check gear mods
    return {
      freeze: false,
      shock: false,
      ignite: false,
      bleed: false,
      poison: false,
      corruptedBlood: false,
      stun: false,
      curse: false,
    };
  }

  countDefenseLayers() {
    const stats = this.build?.stats || {};
    const layers = [];

    if ((stats.Life || 0) > 4000) layers.push("Life Pool");
    if ((stats.EnergyShield || 0) > 500) layers.push("Energy Shield");
    if ((stats.Armour || 0) > 10000) layers.push("Armour");
    if ((stats.Evasion || 0) > 10000) layers.push("Evasion");
    if ((stats.BlockChance || 0) > 40) layers.push("Block");
    if ((stats.SpellBlockChance || 0) > 30) layers.push("Spell Block");
    if ((stats.LifeRegen || 0) > 500) layers.push("Life Regen");

    return layers;
  }

  scoreDefenses(life, resists, armour, block) {
    let score = 0;

    // Life scoring (0-30 points)
    if (life >= 6000) score += 30;
    else if (life >= 5000) score += 25;
    else if (life >= 4000) score += 18;
    else if (life >= 3000) score += 10;
    else score += 5;

    // Resist scoring (0-30 points)
    const resCapped = [resists.fire >= 75, resists.cold >= 75, resists.lightning >= 75];
    score += resCapped.filter(Boolean).length * 8;
    if (resists.chaos >= 0) score += 3;
    if (resists.chaos >= 50) score += 3;

    // Armour scoring (0-20 points)
    if (armour >= 30000) score += 20;
    else if (armour >= 20000) score += 15;
    else if (armour >= 10000) score += 10;
    else if (armour >= 5000) score += 5;

    // Block scoring (0-20 points)
    if (block >= 60) score += 15;
    else if (block >= 40) score += 10;
    else if (block >= 20) score += 5;

    return Math.min(100, score);
  }

  // =============================================
  // OFFENSE ANALYSIS
  // =============================================

  analyzeOffense() {
    const stats = this.build?.stats || {};
    const totalDPS = stats.TotalDPS || stats.FireDotDPS || 0;

    return {
      totalDPS,
      dpsBreakdown: this.getDPSBreakdown(),
      gemAnalysis: this.analyzeGems(),
      dpsTier: this.dpsTier(totalDPS),
      score: this.scoreOffense(totalDPS),
    };
  }

  getDPSBreakdown() {
    const stats = this.build?.stats || {};
    const breakdown = {};

    const dpsStats = [
      "TotalDPS", "FireDotDPS", "PhysicalDPS", "FireDPS",
      "ColdDPS", "LightningDPS", "ChaosDPS",
    ];

    for (const stat of dpsStats) {
      if (stats[stat]) {
        breakdown[stat] = stats[stat];
      }
    }

    return breakdown;
  }

  dpsTier(dps) {
    if (dps >= 10000000) return { tier: "S", label: "God-tier", color: "purple" };
    if (dps >= 5000000) return { tier: "A", label: "Excellent", color: "green" };
    if (dps >= 2000000) return { tier: "B", label: "Good", color: "blue" };
    if (dps >= 1000000) return { tier: "C", label: "Average", color: "yellow" };
    if (dps >= 500000) return { tier: "D", label: "Low", color: "orange" };
    return { tier: "F", label: "Needs work", color: "red" };
  }

  scoreOffense(dps) {
    if (dps >= 10000000) return 100;
    if (dps >= 5000000) return 85;
    if (dps >= 2000000) return 70;
    if (dps >= 1000000) return 55;
    if (dps >= 500000) return 40;
    if (dps >= 100000) return 25;
    return 10;
  }

  // =============================================
  // ITEM ANALYSIS
  // =============================================

  analyzeItems() {
    const activeSet = this.itemSets?.[0];
    if (!activeSet) return [];

    const results = [];

    for (const [slotName, itemId] of Object.entries(activeSet.slots)) {
      const item = this.items.find((i) => i.id === itemId);
      if (!item) continue;

      const analysis = {
        slot: slotName,
        item,
        score: this.scoreItem(item, slotName),
        modTiers: this.analyzeModTiers(item),
        openAffixes: this.detectOpenAffixes(item),
        craftSuggestions: this.suggestCrafts(item, slotName),
        weaknesses: this.findItemWeaknesses(item, slotName),
      };

      results.push(analysis);
    }

    // Sort by score (worst first = highest priority to upgrade)
    results.sort((a, b) => a.score - b.score);

    return results;
  }

  scoreItem(item, slot) {
    let score = 0;
    const maxScore = 100;

    for (const mod of item.mods) {
      for (const stat of mod.stats) {
        score += this.statValue(stat, slot);
      }
    }

    // Normalize to 0-100
    const expected = this.expectedScoreForSlot(slot, this.build?.level || 1);
    return Math.min(100, Math.round((score / expected) * 100));
  }

  statValue(stat, slot) {
    const weights = this.getStatWeights();
    const key = stat.stat?.toLowerCase() || "";

    if (key.includes("maximum life") || key === "life") {
      return (stat.value || 0) * (weights.life || 1);
    }
    if (key.includes("fire resistance")) {
      return (stat.value || 0) * (weights.fireRes || 0.5);
    }
    if (key.includes("cold resistance")) {
      return (stat.value || 0) * (weights.coldRes || 0.5);
    }
    if (key.includes("lightning resistance")) {
      return (stat.value || 0) * (weights.lightRes || 0.5);
    }
    if (key.includes("chaos resistance")) {
      return (stat.value || 0) * (weights.chaosRes || 0.8);
    }
    if (key.includes("armour")) {
      return (stat.value || 0) * (weights.armour || 0.05);
    }
    if (key.includes("movement speed")) {
      return (stat.value || 0) * (weights.moveSpeed || 2);
    }
    if (key.includes("fire skill gems") || key.includes("fire damage")) {
      return (stat.value || 0) * (weights.fireDamage || 10);
    }
    if (key.includes("maximum life") && stat.type === "increased") {
      return (stat.value || 0) * (weights.percentLife || 3);
    }
    if (key.includes("fire resistance") && key.includes("maximum")) {
      return (stat.value || 0) * 15; // max res is very valuable
    }

    return 1; // unknown mod gets minimal value
  }

  getStatWeights() {
    // Weights vary by build type — this is for RF Inquisitor
    const buildType = this.detectBuildType();

    if (buildType === "fire_dot") {
      return {
        life: 1.2, percentLife: 4, fireRes: 0.3, coldRes: 0.5,
        lightRes: 0.5, chaosRes: 0.8, armour: 0.05,
        moveSpeed: 3, fireDamage: 12, dotMulti: 15,
      };
    }

    // Default balanced weights
    return {
      life: 1.0, percentLife: 3, fireRes: 0.5, coldRes: 0.5,
      lightRes: 0.5, chaosRes: 0.7, armour: 0.05,
      moveSpeed: 2, fireDamage: 8, dotMulti: 10,
    };
  }

  detectBuildType() {
    const stats = this.build?.stats || {};
    if (stats.FireDotDPS > 0) return "fire_dot";
    if (stats.ColdDotDPS > 0) return "cold_dot";
    if (stats.PhysicalDPS > 0) return "attack";
    return "spell";
  }

  expectedScoreForSlot(slot, level) {
    // Expected total stat value for a good item at this level
    const base = level * 2;
    const slotMultipliers = {
      "Body Armour": 1.5,
      "Helmet": 1.2,
      "Gloves": 1.0,
      "Boots": 1.1,
      "Belt": 1.0,
      "Amulet": 1.3,
      "Ring 1": 0.9,
      "Ring 2": 0.9,
      "Weapon 1": 1.4,
    };
    return base * (slotMultipliers[slot] || 1.0);
  }

  analyzeModTiers(item) {
    const tiers = [];

    for (const mod of item.explicits || []) {
      for (const stat of mod.stats) {
        const tier = this.estimateModTier(stat);
        tiers.push({ ...stat, tier, raw: mod.raw });
      }
    }

    return tiers;
  }

  estimateModTier(stat) {
    const key = stat.stat?.toLowerCase() || "";
    const val = Math.abs(stat.value || 0);

    if (key.includes("maximum life") && stat.type === "flat") {
      if (val >= 90) return { tier: 1, label: "T1", color: "gold" };
      if (val >= 80) return { tier: 2, label: "T2", color: "green" };
      if (val >= 70) return { tier: 3, label: "T3", color: "blue" };
      if (val >= 60) return { tier: 4, label: "T4", color: "white" };
      return { tier: 5, label: "T5+", color: "gray" };
    }

    if (key.includes("resistance") && stat.type === "percent") {
      if (val >= 42) return { tier: 1, label: "T1", color: "gold" };
      if (val >= 36) return { tier: 2, label: "T2", color: "green" };
      if (val >= 30) return { tier: 3, label: "T3", color: "blue" };
      if (val >= 24) return { tier: 4, label: "T4", color: "white" };
      return { tier: 5, label: "T5+", color: "gray" };
    }

    if (key.includes("movement speed")) {
      if (val >= 30) return { tier: 1, label: "T1", color: "gold" };
      if (val >= 25) return { tier: 2, label: "T2", color: "green" };
      if (val >= 20) return { tier: 3, label: "T3", color: "blue" };
      return { tier: 4, label: "T4+", color: "white" };
    }

    return { tier: 0, label: "??", color: "gray" };
  }

  detectOpenAffixes(item) {
    // PoE items: max 3 prefixes + 3 suffixes (rare)
    // This is a simplified detection
    const prefixes = []; // life, ES, armour, evasion, flat damage
    const suffixes = []; // resists, attributes, crit, speed

    for (const mod of item.explicits || []) {
      const raw = mod.raw?.toLowerCase() || "";
      if (raw.includes("life") || raw.includes("armour") || raw.includes("energy shield")) {
        prefixes.push(mod);
      } else {
        suffixes.push(mod);
      }
    }

    return {
      prefixCount: prefixes.length,
      suffixCount: suffixes.length,
      openPrefixes: Math.max(0, 3 - prefixes.length),
      openSuffixes: Math.max(0, 3 - suffixes.length),
      canCraft: prefixes.length < 3 || suffixes.length < 3,
    };
  }

  suggestCrafts(item, slot) {
    const open = this.detectOpenAffixes(item);
    const suggestions = [];

    if (open.openPrefixes > 0) {
      const hasLife = item.mods.some((m) => m.raw?.toLowerCase().includes("life"));
      if (!hasLife) {
        suggestions.push({
          type: "prefix",
          craft: "+70 to maximum Life (benchcraft)",
          priority: "high",
          reason: "No life roll on item",
        });
      }
    }

    if (open.openSuffixes > 0) {
      const hasMoveSpeed = item.mods.some((m) => m.raw?.toLowerCase().includes("movement speed"));
      if (slot === "Boots" && !hasMoveSpeed) {
        suggestions.push({
          type: "suffix",
          craft: "25% increased Movement Speed (benchcraft)",
          priority: "high",
          reason: "Boots without movement speed",
        });
      }

      // Check if resists need help
      const stats = this.build?.stats || {};
      if ((stats.ColdResist || 0) < 75) {
        suggestions.push({
          type: "suffix",
          craft: "+Cold Resistance (benchcraft)",
          priority: "medium",
          reason: "Cold resist not capped",
        });
      }
    }

    return suggestions;
  }

  findItemWeaknesses(item, slot) {
    const weaknesses = [];

    // No life roll
    const hasLife = item.mods.some((m) =>
      m.stats.some((s) => s.stat?.toLowerCase().includes("life") && s.type === "flat")
    );
    if (!hasLife && item.rarity === "RARE") {
      weaknesses.push({ severity: "high", issue: "No flat life roll" });
    }

    // Low life roll
    const lifeMod = item.mods.find((m) =>
      m.stats.some((s) => s.stat?.toLowerCase().includes("maximum life") && s.type === "flat")
    );
    if (lifeMod) {
      const lifeVal = lifeMod.stats[0]?.value || 0;
      if (lifeVal < 70) {
        weaknesses.push({ severity: "medium", issue: `Low life roll (${lifeVal}, T3+)` });
      }
    }

    // Boots without movement speed
    if (slot === "Boots") {
      const hasMS = item.mods.some((m) => m.raw?.toLowerCase().includes("movement speed"));
      if (!hasMS) {
        weaknesses.push({ severity: "high", issue: "No movement speed" });
      }
    }

    return weaknesses;
  }

  // =============================================
  // GEM ANALYSIS
  // =============================================

  analyzeGems() {
    const allGems = [];
    const skillSets = this.skills || [];

    for (const set of skillSets) {
      for (const skill of set.skills || []) {
        for (const gem of skill.gems || []) {
          const analysis = {
            ...gem,
            slot: skill.slot,
            skillLabel: skill.label,
            canLevelUp: gem.level < 21,
            canQuality: gem.quality < 23,
            qualityImpact: this.gemQualityImpact(gem),
            levelBreakpoint: this.gemLevelBreakpoint(gem),
          };
          allGems.push(analysis);
        }
      }
    }

    return allGems;
  }

  gemQualityImpact(gem) {
    if (gem.quality >= 23) return "maxed";
    if (gem.quality >= 20) return "minor upgrade available (23% via Hillock)";
    return `+${20 - gem.quality}% quality available`;
  }

  gemLevelBreakpoint(gem) {
    if (gem.level >= 21) return "Max level (corrupted 21)";
    if (gem.level >= 20) return "Can corrupt for level 21 (+~10-15% DPS)";
    return `${20 - gem.level} levels to max`;
  }

  // =============================================
  // ISSUE DETECTION
  // =============================================

  detectIssues() {
    const issues = [];
    const stats = this.build?.stats || {};

    // Resist checks
    if ((stats.FireResist || 0) < 75) {
      issues.push({
        severity: "critical",
        category: "defense",
        issue: `Fire Resistance uncapped: ${stats.FireResist || 0}% (need 75%)`,
        fix: `Need +${75 - (stats.FireResist || 0)}% fire resistance`,
      });
    }
    if ((stats.ColdResist || 0) < 75) {
      issues.push({
        severity: "critical",
        category: "defense",
        issue: `Cold Resistance uncapped: ${stats.ColdResist || 0}% (need 75%)`,
        fix: `Need +${75 - (stats.ColdResist || 0)}% cold resistance`,
      });
    }
    if ((stats.LightningResist || 0) < 75) {
      issues.push({
        severity: "critical",
        category: "defense",
        issue: `Lightning Resistance uncapped: ${stats.LightningResist || 0}% (need 75%)`,
        fix: `Need +${75 - (stats.LightningResist || 0)}% lightning resistance`,
      });
    }

    // Chaos res warning
    if ((stats.ChaosResist || 0) < 0) {
      issues.push({
        severity: "warning",
        category: "defense",
        issue: `Negative Chaos Resistance: ${stats.ChaosResist}%`,
        fix: "Add chaos resistance on gear or passives",
      });
    }

    // Life check
    if ((stats.Life || 0) < 4000) {
      issues.push({
        severity: "critical",
        category: "defense",
        issue: `Life pool too low: ${stats.Life} (minimum 4000 recommended)`,
        fix: "Add life on gear, take life nodes on tree",
      });
    } else if ((stats.Life || 0) < 5000) {
      issues.push({
        severity: "warning",
        category: "defense",
        issue: `Life pool could be higher: ${stats.Life} (5000+ recommended for red maps)`,
        fix: "Upgrade life rolls on gear",
      });
    }

    // Overcap check for curse maps
    const overcapFire = (stats.FireResist || 0) - 75;
    const overcapCold = (stats.ColdResist || 0) - 75;
    const overcapLight = (stats.LightningResist || 0) - 75;
    const minOvercap = Math.min(overcapFire, overcapCold, overcapLight);
    if (minOvercap < 24 && minOvercap >= 0) {
      issues.push({
        severity: "info",
        category: "defense",
        issue: `Low resist overcap (${minOvercap}%) — vulnerable to Elemental Weakness curse`,
        fix: "Need +24% overcap on all resists for curse maps",
      });
    }

    // DPS check
    const dps = stats.TotalDPS || stats.FireDotDPS || 0;
    if (dps < 500000) {
      issues.push({
        severity: "warning",
        category: "offense",
        issue: `Low DPS: ${this.formatNumber(dps)}`,
        fix: "Upgrade gem levels, add damage mods on gear",
      });
    }

    // Item-specific issues
    const itemAnalysis = this.analyzeItems();
    for (const ia of itemAnalysis) {
      for (const weakness of ia.weaknesses) {
        issues.push({
          severity: weakness.severity === "high" ? "warning" : "info",
          category: "items",
          issue: `${ia.slot}: ${weakness.issue}`,
          fix: weakness.issue,
        });
      }
    }

    // Sort by severity
    const severityOrder = { critical: 0, warning: 1, info: 2 };
    issues.sort((a, b) => severityOrder[a.severity] - severityOrder[b.severity]);

    return issues;
  }

  // =============================================
  // SUGGESTIONS
  // =============================================

  generateSuggestions() {
    const suggestions = [];
    const stats = this.build?.stats || {};

    // Item upgrade priorities
    const items = this.analyzeItems();
    for (const item of items.slice(0, 3)) { // top 3 worst items
      if (item.score < 60) {
        suggestions.push({
          priority: "high",
          category: "upgrade",
          slot: item.slot,
          message: `Upgrade ${item.slot} (score: ${item.score}/100)`,
          details: item.weaknesses.map((w) => w.issue),
          estimatedImpact: "Significant improvement",
        });
      }
    }

    // Craft suggestions
    for (const item of items) {
      for (const craft of item.craftSuggestions) {
        suggestions.push({
          priority: craft.priority,
          category: "craft",
          slot: item.slot,
          message: `Craft on ${item.slot}: ${craft.craft}`,
          details: [craft.reason],
          estimatedImpact: "Free improvement (benchcraft)",
        });
      }
    }

    // Gem level suggestions
    const gems = this.analyzeGems();
    const upgradableGems = gems.filter((g) => g.level < 20 && g.enabled);
    if (upgradableGems.length > 0) {
      suggestions.push({
        priority: "medium",
        category: "gems",
        message: `${upgradableGems.length} gems not at max level`,
        details: upgradableGems.map((g) => `${g.gemId}: level ${g.level}/20`),
        estimatedImpact: "Easy DPS/utility gain",
      });
    }

    // Corruption suggestions
    const maxGems = gems.filter((g) => g.level === 20 && g.quality === 20);
    if (maxGems.length > 0) {
      suggestions.push({
        priority: "low",
        category: "gems",
        message: `${maxGems.length} gems ready to corrupt (20/20 → 21/20)`,
        details: maxGems.map((g) => `${g.gemId}: corrupt for ~10-15% more damage`),
        estimatedImpact: "Good DPS gain if successful",
      });
    }

    return suggestions;
  }

  // =============================================
  // PROGRESSION
  // =============================================

  assessProgression() {
    const level = this.build?.level || 1;
    const stats = this.build?.stats || {};
    const dps = stats.TotalDPS || stats.FireDotDPS || 0;

    let phase, nextGoals;

    if (level < 70) {
      phase = "Leveling";
      nextGoals = [
        "Cap elemental resistances",
        "Get a 4-link setup",
        "Reach 3000+ life",
        "Complete lab trials",
      ];
    } else if (level < 80) {
      phase = "Early Mapping";
      nextGoals = [
        "Cap all elemental resists",
        "Reach 4500+ life",
        "Get a 5-link or budget 6-link",
        "Get movement speed on boots",
        "Set up CWDT + guard skill",
      ];
    } else if (level < 90) {
      phase = "Mid Mapping";
      nextGoals = [
        "Reach 5500+ life",
        "Get chaos resistance > 0",
        "Reach 1M+ DPS",
        "Complete uber lab",
        "Get ailment immunity plan",
      ];
    } else {
      phase = "Endgame";
      nextGoals = [
        "Min-max item slots",
        "Get all gems to 21/23",
        "Optimize passive tree",
        "Reach 3M+ DPS for pinnacle bosses",
        "Get overcapped resists for curse maps",
      ];
    }

    return {
      phase,
      level,
      dps,
      nextGoals,
      completedGoals: this.checkCompletedGoals(stats),
    };
  }

  checkCompletedGoals(stats) {
    const completed = [];

    if (stats.FireResist >= 75 && stats.ColdResist >= 75 && stats.LightningResist >= 75) {
      completed.push("Elemental resists capped");
    }
    if (stats.Life >= 5000) completed.push("5000+ life reached");
    if (stats.Life >= 6000) completed.push("6000+ life reached");
    if (stats.ChaosResist >= 0) completed.push("Positive chaos resistance");
    if ((stats.TotalDPS || stats.FireDotDPS || 0) >= 1000000) completed.push("1M+ DPS");
    if ((stats.TotalDPS || stats.FireDotDPS || 0) >= 3000000) completed.push("3M+ DPS");

    return completed;
  }

  generateChecklist() {
    const stats = this.build?.stats || {};
    const dps = stats.TotalDPS || stats.FireDotDPS || 0;

    return [
      { label: "Elemental resists capped", done: stats.FireResist >= 75 && stats.ColdResist >= 75 && stats.LightningResist >= 75 },
      { label: "4000+ life", done: (stats.Life || 0) >= 4000 },
      { label: "5000+ life", done: (stats.Life || 0) >= 5000 },
      { label: "6000+ life", done: (stats.Life || 0) >= 6000 },
      { label: "Chaos resist ≥ 0", done: (stats.ChaosResist || 0) >= 0 },
      { label: "Chaos resist ≥ 50", done: (stats.ChaosResist || 0) >= 50 },
      { label: "Movement speed on boots", done: this.hasModOnSlot("Boots", "movement speed") },
      { label: "Guard skill setup", done: this.hasGem("MoltenShell") || this.hasGem("Steelskin") },
      { label: "500K+ DPS", done: dps >= 500000 },
      { label: "1M+ DPS", done: dps >= 1000000 },
      { label: "3M+ DPS", done: dps >= 3000000 },
      { label: "All gems level 20+", done: this.allGemsMaxLevel() },
    ];
  }

  // =============================================
  // HELPERS
  // =============================================

  hasModOnSlot(slotName, modText) {
    const activeSet = this.itemSets?.[0];
    if (!activeSet) return false;
    const itemId = activeSet.slots[slotName];
    const item = this.items.find((i) => i.id === itemId);
    if (!item) return false;
    return item.mods.some((m) => m.raw?.toLowerCase().includes(modText.toLowerCase()));
  }

  hasGem(gemId) {
    for (const set of this.skills || []) {
      for (const skill of set.skills || []) {
        if (skill.gems.some((g) => g.gemId === gemId || g.skillId === gemId)) return true;
      }
    }
    return false;
  }

  allGemsMaxLevel() {
    for (const set of this.skills || []) {
      for (const skill of set.skills || []) {
        for (const gem of skill.gems) {
          if (gem.enabled && gem.level < 20) return false;
        }
      }
    }
    return true;
  }

  calculateOverallScore() {
    const stats = this.build?.stats || {};
    const defScore = this.scoreDefenses(
      stats.Life || 0,
      {
        fire: stats.FireResist || 0,
        cold: stats.ColdResist || 0,
        lightning: stats.LightningResist || 0,
        chaos: stats.ChaosResist || 0,
      },
      stats.Armour || 0,
      stats.BlockChance || 0
    );
    const offScore = this.scoreOffense(stats.TotalDPS || stats.FireDotDPS || 0);

    return Math.round(defScore * 0.5 + offScore * 0.5);
  }

  formatNumber(num) {
    if (num >= 1000000) return (num / 1000000).toFixed(1) + "M";
    if (num >= 1000) return (num / 1000).toFixed(0) + "K";
    return num.toString();
  }
}

export default BuildAnalyzer;
