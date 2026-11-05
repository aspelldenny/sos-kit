//! Comparison prices (USD per 1M tokens). Not a bill: the maintainer pays by subscription, and
//! prices change; `--prices` or `~/.config/sos/prices.toml` overrides the embedded table.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

const EMBEDDED: &str = include_str!("prices.toml");

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Price {
    pub input: f64,
    pub cache_write_5m: f64,
    pub cache_write_1h: f64,
    pub cache_read: f64,
    pub output: f64,
}

#[derive(Deserialize)]
struct File {
    version: String,
    prices: BTreeMap<String, Price>,
}

pub struct Prices {
    pub version: String,
    table: BTreeMap<String, Price>,
}

impl Prices {
    pub fn load(path: Option<&Path>) -> Result<Prices> {
        let user = path.map(Path::to_path_buf).or_else(|| {
            let p = super::home()?.join(".config/sos/prices.toml");
            p.is_file().then_some(p)
        });
        let text = match &user {
            Some(p) => std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?,
            None => EMBEDDED.to_string(),
        };
        // The parser's message quotes the offending source line: never print it (the file is the
        // user's, and anything could be in it). Only where it is.
        let f: File = toml::from_str(&text).map_err(|e| {
            let line = e.span().map(|s| text[..s.start.min(text.len())].matches('\n').count() + 1);
            let at = line.map(|l| format!(" near line {l}")).unwrap_or_default();
            anyhow::anyhow!("the prices table{} is not valid{at}: expected `version = \"…\"` and [prices.\"<model>\"] tables with \
                             input, cache_write_5m, cache_write_1h, cache_read, output (numbers)",
                            user.as_ref().map(|p| format!(" {}", p.display())).unwrap_or_default())
        })?;
        Ok(Prices { version: f.version, table: f.prices })
    }

    /// The exact id first (a table may price `m[1m]` apart from `m`), else the id without a `[1m]` context tag or a `-YYYYMMDD` snapshot date. No
    /// prefix matching: `claude-opus-5-6` is not `claude-opus-5`; an unknown model has no price.
    pub fn get(&self, model: &str) -> Option<&Price> {
        if let Some(p) = self.table.get(model) {
            return Some(p);
        }
        let base = model.strip_suffix("[1m]").unwrap_or(model);
        let b = base.as_bytes();
        let dated = b.len() > 9 && b[b.len() - 9] == b'-' && b[b.len() - 8..].iter().all(u8::is_ascii_digit);
        self.table.get(base).or_else(|| dated.then(|| self.table.get(&base[..base.len() - 9])).flatten())
    }
}

/// Token counts of one call, in the columns the price applies to. Thinking is part of `output`
/// and the TTL split is part of `cache_write`: neither is added again.
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    pub input: u64,
    pub cache_write: u64,
    /// `cache_creation.ephemeral_1h_input_tokens` when the record has the split; the rest of
    /// `cache_write` is priced at the 5-minute rate.
    pub cache_write_1h: u64,
    pub cache_read: u64,
    pub output: u64,
}

pub fn cost(p: &Price, t: &Tokens) -> f64 {
    let w1h = t.cache_write_1h.min(t.cache_write);
    (t.input as f64 * p.input
        + (t.cache_write - w1h) as f64 * p.cache_write_5m
        + w1h as f64 * p.cache_write_1h
        + t.cache_read as f64 * p.cache_read
        + t.output as f64 * p.output)
        / 1e6
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_table_parses_and_matches_only_known_ids() {
        let p = Prices::load(Some(Path::new("/nonexistent"))).err();
        assert!(p.is_some(), "an explicit missing file is an error, not a silent default");
        let p = Prices { version: "t".into(), table: toml::from_str::<File>(EMBEDDED).unwrap().prices };
        assert!(p.get("claude-opus-5-5").is_some());
        assert_eq!(p.get("claude-opus-5-5[1m]"), p.get("claude-opus-5-5"));
        assert!(p.get("gpt-6-astra").is_none());
        assert_eq!(p.get("claude-haiku-4-5-20251001"), p.get("claude-haiku-4-5"));
        assert_ne!(p.get("claude-opus-5"), p.get("claude-opus-5-5"));
        let one = |input: f64| Price { input, cache_write_5m: 0.0, cache_write_1h: 0.0, cache_read: 0.0, output: 0.0 };
        let t = Prices { version: "t".into(), table: [("m".to_string(), one(4.0)), ("m[1m]".to_string(), one(9.0)), ("n[1m]".to_string(), one(7.0))].into() };
        assert_eq!((t.get("m[1m]").map(|p| p.input), t.get("m").map(|p| p.input), t.get("n[1m]").map(|p| p.input)), (Some(9.0), Some(4.0), Some(7.0)));
        for unknown in ["claude-opus-5-6", "claude-opus-50", "claude-opus-5-5-x", "claude-opus-5-2026"] {
            assert!(p.get(unknown).is_none(), "{unknown} must have no price");
        }
    }

    #[test]
    fn a_broken_table_never_echoes_its_content() {
        let dir = std::env::temp_dir().join(format!("sos-prices-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for (i, body) in ["version = \"x\"\n[prices.\"m\"]\ninput = SOS-SECRET-7f3a9c\n",
                          "version = \"x\"\n[prices.\"m\"]\ninput = \"SOS-SECRET-7f3a9c\"\n"].iter().enumerate() {
            let f = dir.join(format!("p{i}.toml"));
            std::fs::write(&f, body).unwrap();
            let e = format!("{:#}", Prices::load(Some(&f)).err().expect("an error"));
            assert!(!e.contains("SOS-SECRET"), "{e}");
            assert!(e.contains("not valid"), "{e}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cost_counts_each_token_once() {
        let p = Price { input: 4.0, cache_write_5m: 5.0, cache_write_1h: 8.0, cache_read: 0.4, output: 20.0 };
        let t = Tokens { input: 1_000_000, cache_write: 2_000_000, cache_write_1h: 500_000, cache_read: 10_000_000, output: 1_000_000 };
        // 4 + 1.5M·5 + 0.5M·8 + 10M·0.4 + 20 = 4 + 7.5 + 4 + 4 + 20
        assert!((cost(&p, &t) - 39.5).abs() < 1e-9, "{}", cost(&p, &t));
    }
}
