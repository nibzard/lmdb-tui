use once_cell::sync::Lazy;
use serde_json::Value;
use std::sync::RwLock;

/// Function signature for decoder plugins.
pub type DecoderFn = Box<dyn Fn(&[u8]) -> Option<Value> + Send + Sync>;

static DECODERS: Lazy<RwLock<Vec<DecoderFn>>> = Lazy::new(|| RwLock::new(Vec::new()));

/// Register a custom decoder.
pub fn register_decoder<F>(f: F)
where
    F: Fn(&[u8]) -> Option<Value> + Send + Sync + 'static,
{
    DECODERS.write().unwrap().push(Box::new(f));
}

/// Decode bytes using registered plugins.
pub fn decode_with_plugins(bytes: &[u8]) -> Option<Value> {
    for dec in DECODERS.read().unwrap().iter() {
        if let Some(v) = dec(bytes) {
            return Some(v);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn custom_decoder_is_used() {
        register_decoder(|b| {
            if b == b"magic" {
                Some(json!({"ok": true}))
            } else {
                None
            }
        });
        let v = decode_with_plugins(b"magic").expect("decoder should succeed");
        assert_eq!(v, json!({"ok": true}));
    }
}
