/**
 * Market Intelligence — price trends, buy timing advisor,
 * league phase detection, and price alert system
 * Data source: poe.ninja API (free, no auth)
 */

const POE_NINJA_BASE = "https://poe.ninja/api/data";

class MarketIntelligence {
  constructor(options = {}) {
    this.league = options.league || "Necropolis";
    this.cache = new Map();
    this.cacheExpiry = options.cacheMinutes || 5;
    this.priceHistory = new Map(); // item -> array of {date, price}
    this.alerts = [];
    this.leagueStartDate = options.leagueStartDate || null;
  }

  // =============================================
  // PRICE FETCHING
  // =============================================

  async fetchItemPrice(itemName, itemType = "unique") {
    const cacheKey = `${itemType}:${itemName}`;
    const cached = this.cache.get(cacheKey);
    if (cached && Date.now() - cached.timestamp < this.cacheExpiry * 60000) {
      return cached.data;
    }

    try {
      const endpoints = {
        unique: `${POE_NINJA_BASE}/itemoverview?league=${this.league}&type=UniqueArmour`,
        uniqueWeapon: `${POE_NINJA_BASE}/itemoverview?league=${this.league}&type=UniqueWeapon`,
        uniqueAccessory: `${POE_NINJA_BASE}/itemoverview?league=${this.league}&type=UniqueAccessory`,
        uniqueFlask: `${POE_NINJA_BASE}/itemoverview?league=${this.league}&type=UniqueFlask`,
        uniqueJewel: `${POE_NINJA_BASE}/itemoverview?league=${this.league}&type=UniqueJewel`,
        currency: `${POE_NINJA_BASE}/currencyoverview?league=${this.league}&type=Currency`,
        gem: `${POE_NINJA_BASE}/itemoverview?league=${this.league}&type=SkillGem`,
      };

      const url = endpoints[itemType] || endpoints.unique;
      const response = await fetch(url);
      const data = await response.json();

      const item = (data.lines || []).find(l =>
        l.name?.toLowerCase() === itemName.toLowerCase()
      );

      if (item) {
        const result = {
          name: item.name,
          chaosValue: item.chaosValue || 0,
          divineValue: item.divineValue || 0,
          change7d: item.sparkline?.totalChange || 0,
          listingCount: item.listingCount || 0,
          icon: item.icon || null,
          sparkline: item.sparkline?.data || [],
        };

        this.cache.set(cacheKey, { data: result, timestamp: Date.now() });
        this._recordHistory(itemName, result.divineValue || result.chaosValue / 200);
        return result;
      }

      return null;
    } catch (err) {
      console.error(`Price fetch failed for ${itemName}:`, err);
      return null;
    }
  }

  async fetchCurrencyRates() {
    const cacheKey = "currency_rates";
    const cached = this.cache.get(cacheKey);
    if (cached && Date.now() - cached.timestamp < this.cacheExpiry * 60000) {
      return cached.data;
    }

    try {
      const response = await fetch(
        `${POE_NINJA_BASE}/currencyoverview?league=${this.league}&type=Currency`
      );
      const data = await response.json();

      const rates = {};
      for (const line of (data.lines || [])) {
        rates[line.currencyTypeName] = {
          chaosEquivalent: line.chaosEquivalent || 0,
          change7d: line.receiveSparkLine?.totalChange || 0,
        };
      }

      this.cache.set(cacheKey, { data: rates, timestamp: Date.now() });
      return rates;
    } catch (err) {
      return {};
    }
  }

  // =============================================
  // PRICE HISTORY TRACKING
  // =============================================

  _recordHistory(itemName, price) {
    if (!this.priceHistory.has(itemName)) {
      this.priceHistory.set(itemName, []);
    }
    const history = this.priceHistory.get(itemName);
    const today = new Date().toISOString().split("T")[0];

    // Only record one entry per day
    const lastEntry = history[history.length - 1];
    if (!lastEntry || lastEntry.date !== today) {
      history.push({ date: today, price, timestamp: Date.now() });
    } else {
      lastEntry.price = price; // Update today's price
    }

    // Keep max 90 days
    if (history.length > 90) history.shift();
  }

  getPriceHistory(itemName, days = 14) {
    const history = this.priceHistory.get(itemName) || [];
    return history.slice(-days);
  }

  // =============================================
  // TREND ANALYSIS
  // =============================================

  calculateTrend(itemName) {
    const history = this.getPriceHistory(itemName, 7);
    if (history.length < 2) return { direction: "unknown", magnitude: 0, confidence: "low" };

    const oldest = history[0].price;
    const newest = history[history.length - 1].price;
    const change = ((newest - oldest) / oldest) * 100;

    let direction, description;
    if (change < -20) { direction = "dropping_fast"; description = "Price crashing rapidly"; }
    else if (change < -5) { direction = "dropping_slow"; description = "Price declining steadily"; }
    else if (change > 20) { direction = "rising_fast"; description = "Price surging"; }
    else if (change > 5) { direction = "rising_slow"; description = "Price climbing gradually"; }
    else { direction = "stable"; description = "Price stable"; }

    // Calculate volatility
    let sumDiff = 0;
    for (let i = 1; i < history.length; i++) {
      sumDiff += Math.abs(history[i].price - history[i-1].price);
    }
    const volatility = history.length > 1 ? sumDiff / (history.length - 1) : 0;

    return {
      direction,
      description,
      change7d: Math.round(change * 10) / 10,
      currentPrice: newest,
      previousPrice: oldest,
      volatility: Math.round(volatility * 100) / 100,
      confidence: history.length >= 5 ? "high" : history.length >= 3 ? "medium" : "low",
      dataPoints: history.length,
    };
  }

  // =============================================
  // LEAGUE PHASE DETECTION
  // =============================================

  detectLeaguePhase() {
    if (!this.leagueStartDate) {
      return { phase: 3, name: "Mid League", daysSinceStart: null };
    }

    const daysSinceStart = Math.floor(
      (Date.now() - new Date(this.leagueStartDate).getTime()) / (1000 * 60 * 60 * 24)
    );

    if (daysSinceStart <= 3) {
      return {
        phase: 1, name: "Launch Frenzy", daysSinceStart,
        description: "Chaos orbs are valuable. Unique prices wildly inflated. DO NOT buy expensive uniques yet.",
        advice: [
          "Sell any valuable uniques immediately — prices drop 80% in 3 days",
          "Hoard chaos orbs — they have maximum purchasing power now",
          "Buy only essential leveling gear (Goldrim, Tabula)",
          "Focus on getting to maps fast — early access = early profit",
        ],
      };
    }

    if (daysSinceStart <= 7) {
      return {
        phase: 2, name: "Crash Period", daysSinceStart,
        description: "Prices crashing daily. Divine orb value establishing. Good time to buy leveling gear.",
        advice: [
          "Buy leveling uniques now — 90% cheaper than day 1",
          "Start saving divine orbs — economy shifting to divine-based",
          "Sell essences and fragments — they're at peak demand",
          "Map sustain items sell well (scarabs, sextants)",
        ],
      };
    }

    if (daysSinceStart <= 21) {
      return {
        phase: 3, name: "Stabilization", daysSinceStart,
        description: "Prices settling. Best time for mid-tier upgrades. Meta builds established.",
        advice: [
          "Good time to buy mid-tier upgrades (5-20 div range)",
          "Build-defining uniques at reasonable prices",
          "Watch poe.ninja for underpriced items",
          "Craft vs buy calculations are most accurate now",
        ],
      };
    }

    if (daysSinceStart <= 42) {
      return {
        phase: 4, name: "Peak Economy", daysSinceStart,
        description: "Economy mature. Best prices for endgame items. Player count starting to decline.",
        advice: [
          "BUY endgame items now — best value in the league",
          "Mirror-tier crafts becoming available",
          "Good time to try expensive builds",
          "Start of best time to buy Mageblood/Headhunter",
        ],
      };
    }

    return {
      phase: 5, name: "Late League", daysSinceStart,
      description: "Player count dropping. Some prices rising (less supply), some falling (less demand).",
      advice: [
        "Niche items may become expensive (fewer sellers)",
        "Common items very cheap (lots of stock, few buyers)",
        "Good time to experiment with off-meta builds",
        "Consider items that hold value in Standard",
      ],
    };
  }

  // =============================================
  // BUY RECOMMENDATION
  // =============================================

  generateBuyRecommendation(itemName) {
    const trend = this.calculateTrend(itemName);
    const phase = this.detectLeaguePhase();
    const history = this.getPriceHistory(itemName, 14);

    let action, reason, urgency, confidence;

    // Phase 1-2: Almost always wait
    if (phase.phase <= 2 && trend.direction !== "rising_fast") {
      action = "WAIT";
      reason = `Early league — ${itemName} price will drop significantly. Current: ${trend.currentPrice?.toFixed(1)}d, expected to drop 30-50% by week 2.`;
      urgency = "none";
      confidence = "high";
    }
    // Dropping fast in any phase
    else if (trend.direction === "dropping_fast") {
      action = "WAIT";
      reason = `Price dropping rapidly (-${Math.abs(trend.change7d)}% in 7 days). Wait for stabilization before buying.`;
      urgency = "none";
      confidence = "high";
    }
    // Dropping slowly in stable phase
    else if (trend.direction === "dropping_slow" && phase.phase >= 3) {
      action = "BUY_SOON";
      reason = `Price declining slowly — approaching league-low. Good entry point within 2-3 days.`;
      urgency = "low";
      confidence = "medium";
    }
    // Rising in late phase
    else if (trend.direction === "rising_slow" && phase.phase >= 4) {
      action = "BUY_NOW";
      reason = `Price rising as supply decreases (late league). Will likely continue climbing. Buy before it gets more expensive.`;
      urgency = "high";
      confidence = "medium";
    }
    // Rising fast
    else if (trend.direction === "rising_fast") {
      action = "BUY_NOW_OR_WAIT";
      reason = `Price surging (+${trend.change7d}% in 7 days). Either buy immediately before further increase, or wait for the spike to settle — prices often correct after sharp rises.`;
      urgency = "medium";
      confidence = "low";
    }
    // Stable in good phase
    else if (trend.direction === "stable" && phase.phase >= 3) {
      action = "BUY_WHEN_READY";
      reason = `Price stable — no advantage to timing. Buy whenever you have the currency.`;
      urgency = "none";
      confidence = "high";
    }
    // Default
    else {
      action = "MONITOR";
      reason = `Unclear trend. Set a price alert and wait for a better signal.`;
      urgency = "none";
      confidence = "low";
    }

    return {
      item: itemName,
      action,
      reason,
      urgency,
      confidence,
      currentPrice: trend.currentPrice,
      trend: trend.direction,
      change7d: trend.change7d,
      leaguePhase: phase.name,
      sparkline: history.map(h => h.price),
    };
  }

  // =============================================
  // BUILD VALUE CALCULATOR
  // =============================================

  async calculateBuildValue(items) {
    let total = 0;
    const breakdown = [];

    for (const item of items) {
      let price = null;

      if (item.rarity === "UNIQUE") {
        const priceData = await this.fetchItemPrice(item.name, "unique");
        if (priceData) {
          price = priceData.divineValue || priceData.chaosValue / 200;
        }
      }

      // For rares, estimate based on mod quality
      if (!price && item.score) {
        price = this._estimateRarePrice(item);
      }

      if (price) {
        total += price;
        breakdown.push({
          slot: item.slot,
          name: item.name,
          price: Math.round(price * 10) / 10,
          source: item.rarity === "UNIQUE" ? "poe.ninja" : "estimated",
        });
      }
    }

    breakdown.sort((a, b) => b.price - a.price);

    return {
      totalValue: Math.round(total * 10) / 10,
      breakdown,
      currency: "divine",
    };
  }

  _estimateRarePrice(item) {
    // Rough estimation based on item score
    // Real version would check mod tiers against trade
    const score = item.score || 50;
    if (score >= 90) return 15;
    if (score >= 80) return 8;
    if (score >= 70) return 4;
    if (score >= 60) return 2;
    if (score >= 50) return 1;
    return 0.5;
  }

  // =============================================
  // UPGRADE SHOPPING
  // =============================================

  async findUpgrades(currentItem, budget, requirements) {
    // This would query the PoE trade API
    // For now, returns structure of what it would return
    return {
      slot: currentItem.slot,
      currentScore: currentItem.score,
      budget,
      results: [],
      searchUrl: this._buildTradeUrl(currentItem, requirements),
    };
  }

  _buildTradeUrl(item, requirements) {
    // Build a pathofexile.com/trade search URL
    const base = "https://www.pathofexile.com/trade/search/" + this.league;
    // Would build query params based on requirements
    return base;
  }

  // =============================================
  // PRICE ALERTS
  // =============================================

  addAlert(alert) {
    this.alerts.push({
      id: Date.now(),
      created: new Date(),
      triggered: false,
      ...alert,
    });
    return this.alerts[this.alerts.length - 1];
  }

  removeAlert(alertId) {
    this.alerts = this.alerts.filter(a => a.id !== alertId);
  }

  async checkAlerts() {
    const triggered = [];

    for (const alert of this.alerts) {
      if (alert.triggered) continue;

      if (alert.type === "price_below") {
        const price = await this.fetchItemPrice(alert.item, alert.itemType);
        if (price && price.divineValue <= alert.threshold) {
          alert.triggered = true;
          triggered.push({
            ...alert,
            currentPrice: price.divineValue,
            message: `${alert.item} dropped to ${price.divineValue}d (target: ${alert.threshold}d)`,
          });
        }
      }

      if (alert.type === "price_change") {
        const trend = this.calculateTrend(alert.item);
        if (Math.abs(trend.change7d) >= alert.threshold) {
          alert.triggered = true;
          triggered.push({
            ...alert,
            change: trend.change7d,
            message: `${alert.item} price changed ${trend.change7d}% (threshold: ${alert.threshold}%)`,
          });
        }
      }
    }

    return triggered;
  }

  getActiveAlerts() {
    return this.alerts.filter(a => !a.triggered);
  }

  // =============================================
  // CRAFT VS BUY
  // =============================================

  compareCraftVsBuy(targetItem, craftingMethod) {
    const buyPrice = this._estimateRarePrice(targetItem);

    const craftCosts = {
      essence: { avgAttempts: 30, costPerAttempt: 0.05, avgTotal: 1.5, worstCase: 5 },
      fossil: { avgAttempts: 20, costPerAttempt: 0.2, avgTotal: 4, worstCase: 15 },
      altRegal: { avgAttempts: 200, costPerAttempt: 0.01, avgTotal: 2, worstCase: 8 },
      harvest: { avgAttempts: 10, costPerAttempt: 0.5, avgTotal: 5, worstCase: 20 },
    };

    const method = craftCosts[craftingMethod] || craftCosts.essence;

    return {
      buyPrice,
      craftAverage: method.avgTotal,
      craftWorstCase: method.worstCase,
      recommendation: method.avgTotal < buyPrice * 0.7
        ? "CRAFT — significantly cheaper on average"
        : method.avgTotal < buyPrice
          ? "CRAFT if you can handle variance, BUY for safety"
          : "BUY — crafting is not cost-efficient",
      riskAssessment: method.worstCase > buyPrice * 2
        ? "HIGH RISK — worst case costs 2x+ the buy price"
        : "LOW RISK — worst case is manageable",
    };
  }
}

module.exports = MarketIntelligence;
