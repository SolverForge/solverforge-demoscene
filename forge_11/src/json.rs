// A small JSON reader for the capture bundle: objects keep their key order,
// numbers stay f64, strings are fully unescaped (\uXXXX and surrogate pairs
// included). Hand-written so the demo depends on platform I/O only.
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub at: usize,
    pub what: &'static str,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON error at byte {}: {}", self.at, self.what)
    }
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(fields) => fields.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Num(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_arr(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_obj(&self) -> Option<&[(String, Json)]> {
        match self {
            Json::Obj(o) => Some(o),
            _ => None,
        }
    }

    /// A scalar as display text (labels may be numbers in a capture).
    pub fn text(&self) -> Option<String> {
        match self {
            Json::Str(s) => Some(s.clone()),
            Json::Num(n) if n.fract() == 0.0 && n.abs() < 1e15 => Some(format!("{}", *n as i64)),
            Json::Num(n) => Some(format!("{n}")),
            Json::Bool(b) => Some(b.to_string()),
            _ => None,
        }
    }
}

pub fn parse(src: &str) -> Result<Json, ParseError> {
    let mut p = Parser {
        s: src.as_bytes(),
        i: 0,
    };
    p.ws();
    let v = p.value(0)?;
    p.ws();
    if p.i != p.s.len() {
        return Err(p.err("trailing data"));
    }
    Ok(v)
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn err(&self, what: &'static str) -> ParseError {
        ParseError { at: self.i, what }
    }

    fn ws(&mut self) {
        while let Some(b' ' | b'\t' | b'\n' | b'\r') = self.s.get(self.i) {
            self.i += 1;
        }
    }

    fn eat(&mut self, lit: &[u8]) -> bool {
        if self.s[self.i..].starts_with(lit) {
            self.i += lit.len();
            true
        } else {
            false
        }
    }

    fn value(&mut self, depth: usize) -> Result<Json, ParseError> {
        if depth > 64 {
            return Err(self.err("nested too deep"));
        }
        match self.s.get(self.i) {
            None => Err(self.err("unexpected end")),
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b't') if self.eat(b"true") => Ok(Json::Bool(true)),
            Some(b'f') if self.eat(b"false") => Ok(Json::Bool(false)),
            Some(b'n') if self.eat(b"null") => Ok(Json::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(self.err("unexpected character")),
        }
    }

    fn object(&mut self, depth: usize) -> Result<Json, ParseError> {
        self.i += 1;
        let mut fields = Vec::new();
        self.ws();
        if self.eat(b"}") {
            return Ok(Json::Obj(fields));
        }
        loop {
            self.ws();
            if self.s.get(self.i) != Some(&b'"') {
                return Err(self.err("expected key"));
            }
            let key = self.string()?;
            self.ws();
            if !self.eat(b":") {
                return Err(self.err("expected ':'"));
            }
            self.ws();
            let v = self.value(depth + 1)?;
            fields.push((key, v));
            self.ws();
            if self.eat(b",") {
                continue;
            }
            if self.eat(b"}") {
                return Ok(Json::Obj(fields));
            }
            return Err(self.err("expected ',' or '}'"));
        }
    }

    fn array(&mut self, depth: usize) -> Result<Json, ParseError> {
        self.i += 1;
        let mut items = Vec::new();
        self.ws();
        if self.eat(b"]") {
            return Ok(Json::Arr(items));
        }
        loop {
            self.ws();
            items.push(self.value(depth + 1)?);
            self.ws();
            if self.eat(b",") {
                continue;
            }
            if self.eat(b"]") {
                return Ok(Json::Arr(items));
            }
            return Err(self.err("expected ',' or ']'"));
        }
    }

    fn hex4(&mut self) -> Result<u32, ParseError> {
        let h = self
            .s
            .get(self.i..self.i + 4)
            .and_then(|b| std::str::from_utf8(b).ok())
            .and_then(|s| u32::from_str_radix(s, 16).ok())
            .ok_or_else(|| self.err("bad \\u escape"))?;
        self.i += 4;
        Ok(h)
    }

    fn string(&mut self) -> Result<String, ParseError> {
        self.i += 1;
        let mut out = Vec::new();
        loop {
            let Some(&b) = self.s.get(self.i) else {
                return Err(self.err("unterminated string"));
            };
            self.i += 1;
            match b {
                b'"' => break,
                b'\\' => {
                    let Some(&e) = self.s.get(self.i) else {
                        return Err(self.err("unterminated escape"));
                    };
                    self.i += 1;
                    let c = match e {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => {
                            let hi = self.hex4()?;
                            let code = if (0xD800..0xDC00).contains(&hi) {
                                if !self.eat(b"\\u") {
                                    return Err(self.err("lone surrogate"));
                                }
                                let lo = self.hex4()?;
                                0x10000 + ((hi - 0xD800) << 10) + (lo.wrapping_sub(0xDC00) & 0x3FF)
                            } else {
                                hi
                            };
                            char::from_u32(code).unwrap_or('?')
                        }
                        _ => return Err(self.err("bad escape")),
                    };
                    let mut buf = [0u8; 4];
                    out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
                }
                _ => out.push(b),
            }
        }
        String::from_utf8(out).map_err(|_| self.err("invalid UTF-8"))
    }

    fn number(&mut self) -> Result<Json, ParseError> {
        let start = self.i;
        while let Some(b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') = self.s.get(self.i) {
            self.i += 1;
        }
        std::str::from_utf8(&self.s[start..self.i])
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .map(Json::Num)
            .ok_or_else(|| self.err("bad number"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_values_in_order() {
        let v = parse(r#"{"b": [1, -2.5e1, true, null], "a": {"x": "y"}}"#).unwrap();
        let o = v.as_obj().unwrap();
        assert_eq!(o[0].0, "b");
        assert_eq!(
            v.get("b").unwrap().as_arr().unwrap()[1].as_f64(),
            Some(-25.0)
        );
        assert_eq!(v.get("a").unwrap().get("x").unwrap().as_str(), Some("y"));
    }

    #[test]
    fn unescapes_strings() {
        let v = parse(r#""a\n\"b\" \u00e9 \ud83d\ude00 \/""#).unwrap();
        assert_eq!(v.as_str(), Some("a\n\"b\" é 😀 /"));
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse("{\"a\": }").is_err());
        assert!(parse("[1, 2").is_err());
        assert!(parse("{} x").is_err());
    }
}
