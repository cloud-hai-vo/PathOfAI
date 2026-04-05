/// Item Image Resolver — Algorithm 51.
/// Resolution cascade: memory cache → disk cache → CDN URL → placeholder.
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const CDN_BASE: &str = "https://web.poecdn.com/image";
const PLACEHOLDER: &str = "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' width='48' height='48'><rect width='48' height='48' fill='%23333' rx='4'/><text x='24' y='30' text-anchor='middle' fill='%23888' font-size='10'>?</text></svg>";

/// Resolved image — either a local file path or a CDN URL.
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedImage {
    LocalFile(PathBuf),
    CdnUrl(String),
    Placeholder,
}

impl ResolvedImage {
    /// Returns the URL string suitable for an `<img src="...">` attribute.
    pub fn to_url(&self) -> String {
        match self {
            ResolvedImage::LocalFile(p) => format!("file://{}", p.display()),
            ResolvedImage::CdnUrl(u)   => u.clone(),
            ResolvedImage::Placeholder => PLACEHOLDER.to_string(),
        }
    }
}

/// Image request — describes what we need a URL for.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImageRequest {
    UniqueItem(String),   // unique item name, e.g. "Kaom's Heart"
    BaseType(String),     // base type, e.g. "Iron Gauntlets"
    Gem(String),          // gem name, e.g. "Righteous Fire"
    Currency(String),     // currency name, e.g. "Divine Orb"
    Placeholder,
}

impl ImageRequest {
    pub fn cache_key(&self) -> String {
        match self {
            ImageRequest::UniqueItem(n) => format!("unique_{}", slug(n)),
            ImageRequest::BaseType(n)   => format!("base_{}", slug(n)),
            ImageRequest::Gem(n)        => format!("gem_{}", slug(n)),
            ImageRequest::Currency(n)   => format!("currency_{}", slug(n)),
            ImageRequest::Placeholder   => "placeholder".to_string(),
        }
    }
}

fn slug(s: &str) -> String {
    s.to_lowercase().replace(' ', "_").replace(['\'', '"', '.'], "")
}

/// In-memory + disk image resolver.
pub struct ImageResolver {
    memory:    HashMap<String, ResolvedImage>,
    cache_dir: PathBuf,
}

impl ImageResolver {
    pub fn new(cache_dir: &Path) -> Self {
        ImageResolver {
            memory:    HashMap::new(),
            cache_dir: cache_dir.to_path_buf(),
        }
    }

    /// Resolve an image request.
    /// Steps 1-2 are synchronous (cache lookup).
    /// Step 3 returns a CDN URL without downloading (download happens async separately).
    pub fn resolve(&mut self, request: &ImageRequest) -> ResolvedImage {
        if *request == ImageRequest::Placeholder {
            return ResolvedImage::Placeholder;
        }

        let key = request.cache_key();

        // 1. Memory cache
        if let Some(img) = self.memory.get(&key) {
            return img.clone();
        }

        // 2. Disk cache
        let disk_path = self.cache_dir.join(format!("{key}.png"));
        if disk_path.exists() {
            let img = ResolvedImage::LocalFile(disk_path);
            self.memory.insert(key.clone(), img.clone());
            return img;
        }

        // 3. CDN URL lookup
        let cdn_url = self.resolve_cdn_url(request);
        let img = ResolvedImage::CdnUrl(cdn_url);
        self.memory.insert(key, img.clone());
        img
    }

    /// Warm the memory cache with a known local file (used after async download completes).
    pub fn warm(&mut self, request: &ImageRequest, local_path: PathBuf) {
        let key = request.cache_key();
        self.memory.insert(key, ResolvedImage::LocalFile(local_path));
    }

    fn resolve_cdn_url(&self, request: &ImageRequest) -> String {
        match request {
            ImageRequest::UniqueItem(name) => {
                UNIQUE_ITEM_IMAGES.get(name.as_str())
                    .map(|p| format!("{CDN_BASE}/{p}"))
                    .unwrap_or_else(|| fallback_cdn_url(name, "unique"))
            }
            ImageRequest::BaseType(name) => {
                BASE_TYPE_IMAGES.get(name.as_str())
                    .map(|p| format!("{CDN_BASE}/{p}"))
                    .unwrap_or_else(|| fallback_cdn_url(name, "base"))
            }
            ImageRequest::Gem(name) => {
                GEM_IMAGES.get(name.as_str())
                    .map(|p| format!("{CDN_BASE}/{p}"))
                    .unwrap_or_else(|| fallback_cdn_url(name, "gem"))
            }
            ImageRequest::Currency(name) => {
                CURRENCY_IMAGES.get(name.as_str())
                    .map(|p| format!("{CDN_BASE}/{p}"))
                    .unwrap_or_else(|| fallback_cdn_url(name, "currency"))
            }
            ImageRequest::Placeholder => PLACEHOLDER.to_string(),
        }
    }
}

fn fallback_cdn_url(name: &str, category: &str) -> String {
    // PoE CDN naming convention fallback
    let slugged = name.to_lowercase().replace(' ', "").replace('\'', "");
    format!("{CDN_BASE}/Art/2DItems/{category}/{slugged}.png")
}

// ── Static lookup tables (selected well-known items) ─────────────────────────

static UNIQUE_ITEM_IMAGES: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "Kaom's Heart"          => "Art/2DItems/Armours/BodyArmours/KaomsHeart.png",
    "Watcher's Eye"         => "Art/2DItems/Jewels/WatchersEye.png",
    "Mageblood"             => "Art/2DItems/Belts/Mageblood.png",
    "Bottled Faith"         => "Art/2DItems/Flasks/BottledFaith.png",
    "Aegis Aurora"          => "Art/2DItems/Armours/Shields/AegisAurora.png",
    "Ashes of the Stars"    => "Art/2DItems/Amulets/AshesOfTheStars.png",
    "Melding of the Flesh"  => "Art/2DItems/Jewels/MeldingOfTheFlesh.png",
};

static GEM_IMAGES: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "Righteous Fire"        => "Art/2DItems/Gems/RighteousFire.png",
    "Scorching Ray"         => "Art/2DItems/Gems/ScorchingRay.png",
    "Fireball"              => "Art/2DItems/Gems/Fireball.png",
    "Arc"                   => "Art/2DItems/Gems/Arc.png",
    "Determination"         => "Art/2DItems/Gems/Determination.png",
    "Vitality"              => "Art/2DItems/Gems/Vitality.png",
};

static CURRENCY_IMAGES: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "Divine Orb"            => "Art/2DItems/Currency/CurrencyImproveMagicItem.png",
    "Chaos Orb"             => "Art/2DItems/Currency/CurrencyRerollRare.png",
    "Exalted Orb"           => "Art/2DItems/Currency/CurrencyAddModToRare.png",
    "Orb of Alteration"     => "Art/2DItems/Currency/CurrencyRerollMagic.png",
    "Orb of Fusing"         => "Art/2DItems/Currency/CurrencyRerollSocketLinks.png",
};

static BASE_TYPE_IMAGES: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "Iron Gauntlets"        => "Art/2DItems/Armours/Gloves/GlovesAtlasStr1.png",
    "Full Dragonscale"      => "Art/2DItems/Armours/BodyArmours/BodyStrDex7.png",
};

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn resolver() -> (ImageResolver, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        (ImageResolver::new(dir.path()), dir)
    }

    #[test]
    fn placeholder_resolves_to_placeholder() {
        let (mut r, _d) = resolver();
        let img = r.resolve(&ImageRequest::Placeholder);
        assert_eq!(img, ResolvedImage::Placeholder);
    }

    #[test]
    fn known_unique_resolves_to_cdn_url() {
        let (mut r, _d) = resolver();
        let img = r.resolve(&ImageRequest::UniqueItem("Kaom's Heart".to_string()));
        match &img {
            ResolvedImage::CdnUrl(u) => assert!(u.contains("KaomsHeart"), "expected Kaom's Heart URL, got {u}"),
            other => panic!("expected CdnUrl, got {other:?}"),
        }
    }

    #[test]
    fn unknown_unique_falls_back_to_cdn_convention() {
        let (mut r, _d) = resolver();
        let img = r.resolve(&ImageRequest::UniqueItem("Some Obscure Item".to_string()));
        match &img {
            ResolvedImage::CdnUrl(u) => assert!(u.starts_with(CDN_BASE), "should use CDN base: {u}"),
            other => panic!("expected CdnUrl, got {other:?}"),
        }
    }

    #[test]
    fn memory_cache_returns_same_result_on_second_call() {
        let (mut r, _d) = resolver();
        let req = ImageRequest::UniqueItem("Mageblood".to_string());
        let first  = r.resolve(&req);
        let second = r.resolve(&req);
        assert_eq!(first, second, "memory cache should return same result");
    }

    #[test]
    fn disk_cached_file_resolves_to_local_file() {
        let dir = tempdir().unwrap();
        let mut r = ImageResolver::new(dir.path());
        // Pre-create a disk-cached file
        let req = ImageRequest::UniqueItem("Mageblood".to_string());
        let key = req.cache_key();
        let disk_path = dir.path().join(format!("{key}.png"));
        std::fs::write(&disk_path, b"fake png data").unwrap();

        let img = r.resolve(&req);
        match img {
            ResolvedImage::LocalFile(p) => assert_eq!(p, disk_path),
            other => panic!("expected LocalFile from disk cache, got {other:?}"),
        }
    }

    #[test]
    fn warm_updates_memory_cache_to_local_file() {
        let dir = tempdir().unwrap();
        let mut r = ImageResolver::new(dir.path());
        let req = ImageRequest::Gem("Righteous Fire".to_string());
        let local = dir.path().join("gem_righteousfire.png");
        std::fs::write(&local, b"").unwrap();

        r.warm(&req, local.clone());
        let img = r.resolve(&req);
        assert_eq!(img, ResolvedImage::LocalFile(local));
    }

    #[test]
    fn known_gem_resolves_to_cdn_url() {
        let (mut r, _d) = resolver();
        let img = r.resolve(&ImageRequest::Gem("Righteous Fire".to_string()));
        match &img {
            ResolvedImage::CdnUrl(u) => assert!(u.contains("RighteousFire"), "got {u}"),
            other => panic!("expected CdnUrl, got {other:?}"),
        }
    }

    #[test]
    fn cache_key_slugs_apostrophes_and_spaces() {
        let key = ImageRequest::UniqueItem("Kaom's Heart".to_string()).cache_key();
        assert!(!key.contains('\''), "key should not contain apostrophes");
        assert!(!key.contains(' '),  "key should not contain spaces");
    }

    #[test]
    fn to_url_local_file_produces_file_scheme() {
        let img = ResolvedImage::LocalFile(PathBuf::from("/some/path/item.png"));
        assert!(img.to_url().starts_with("file://"), "expected file:// URL");
    }

    #[test]
    fn to_url_cdn_returns_cdn_string() {
        let img = ResolvedImage::CdnUrl("https://web.poecdn.com/image/foo.png".to_string());
        assert_eq!(img.to_url(), "https://web.poecdn.com/image/foo.png");
    }
}
