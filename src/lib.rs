use error_chain::error_chain;
use jq_rs;
use serde_json;

error_chain! {}

pub trait JQ: Sized {
    fn jq(&self, query: &str) -> Result<Self>;
}

impl JQ for serde_json::Value {
    fn jq(&self, query: &str) -> Result<Self> {
        let mut query = jq_rs::compile(query)
            .chain_err(|| format!("Couldn't compile {} as a JQ query", query))?;
        let json = query
            .run(&serde_json::to_string(&self).unwrap())
            .chain_err(|| "JQ failed")?;
        serde_json::from_str(&json).chain_err(|| "Couldn't re-serialize")
    }
}

pub trait PromFormat {
    fn prom_format(&self) -> String;
}

impl PromFormat for serde_json::Value {
    fn prom_format(&self) -> String {
        let mut s = String::new();
        if let Self::Object(map) = self {
            for (key, value) in map.iter() {
                s.push_str(&match value {
                    Self::Bool(b) => format!("{} {}\n", key, *b as i32),
                    Self::Number(n) => format!("{} {}\n", key, n),
                    _ => String::new(),
                })
            }
        }
        s
    }
}
