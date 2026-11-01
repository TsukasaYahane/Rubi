//! Playing an interactive video: the current node, its hidden variables, and which choices
//! show. Conditions and actions are small expressions over `$variables`:
//! `$a>=2&&$b==1`, `$a=$a+1;$b=0`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rubi_api::{SteinApi, SteinChoice, SteinNode};

use crate::error::Result;

/// One interactive video being played.
pub struct SteinSession {
    api: Arc<dyn SteinApi>,
    bvid: String,
    graph_version: u64,
    state: Mutex<State>,
}

struct State {
    node: SteinNode,
    vars: HashMap<String, f64>,
}

impl SteinSession {
    /// `None` when the video is not interactive.
    pub async fn open(api: Arc<dyn SteinApi>, bvid: &str, cid: u64) -> Result<Option<Self>> {
        let Some(graph_version) = api.graph_version(bvid, cid).await? else { return Ok(None) };
        let node = api.node(bvid, graph_version, None).await?;
        let vars = node.vars.iter().map(|v| (v.id.clone(), v.value)).collect();
        Ok(Some(Self { api, bvid: bvid.to_owned(), graph_version, state: Mutex::new(State { node, vars }) }))
    }

    /// The current node with only the choices its conditions allow.
    pub fn current(&self) -> SteinNode {
        let s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        visible(&s.node, &s.vars)
    }

    /// Variables the video marks as visible, with their current values.
    pub fn shown_vars(&self) -> Vec<(String, f64)> {
        let s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        s.node
            .vars
            .iter()
            .filter(|v| v.shown)
            .map(|v| (v.name.clone(), s.vars.get(&v.id).copied().unwrap_or(v.value)))
            .collect()
    }

    /// Takes the choice leading to `edge_id`: runs its action, then moves to that node.
    pub async fn choose(&self, edge_id: u64) -> Result<SteinNode> {
        let action = {
            let s = self.state.lock().unwrap_or_else(|e| e.into_inner());
            s.node.questions.iter().flat_map(|q| &q.choices).find(|c| c.edge_id == edge_id).map(|c| c.action.clone())
        };
        let next = self.api.node(&self.bvid, self.graph_version, Some(edge_id)).await?;
        let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(a) = action {
            apply_action(&a, &mut s.vars);
        }
        // Variables new to this node start at the server's value.
        for v in &next.vars {
            s.vars.entry(v.id.clone()).or_insert(v.value);
        }
        s.node = next;
        Ok(visible(&s.node, &s.vars))
    }
}

fn visible(node: &SteinNode, vars: &HashMap<String, f64>) -> SteinNode {
    let mut n = node.clone();
    for q in &mut n.questions {
        q.choices.retain(|c: &SteinChoice| !c.is_hidden && eval_condition(&c.condition, vars));
    }
    n
}

/// Whether `expr` holds. Empty or unreadable conditions count as true, so a choice is never
/// lost to a syntax this evaluator does not know.
pub fn eval_condition(expr: &str, vars: &HashMap<String, f64>) -> bool {
    if expr.trim().is_empty() {
        return true;
    }
    Parser::new(expr, vars).expression().map(|v| v != 0.0).unwrap_or(true)
}

/// Runs `$x=expr` statements separated by `;`. Unreadable statements are skipped.
pub fn apply_action(action: &str, vars: &mut HashMap<String, f64>) {
    for stmt in action.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        let Some((lhs, rhs)) = stmt.split_once('=') else { continue };
        let name = lhs.trim();
        if !name.starts_with('$') || rhs.starts_with('=') {
            continue;
        }
        if let Some(v) = Parser::new(rhs, vars).expression() {
            vars.insert(name.to_owned(), v);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Var(String),
    Op(&'static str),
}

/// Precedence climbing over `|| && == != > >= < <= + - * /`, parentheses and unary minus.
struct Parser {
    toks: Vec<Tok>,
    pos: usize,
    vars: HashMap<String, f64>,
}

impl Parser {
    fn new(src: &str, vars: &HashMap<String, f64>) -> Self {
        Self { toks: tokenize(src), pos: 0, vars: vars.clone() }
    }

    fn expression(&mut self) -> Option<f64> {
        let v = self.or()?;
        (self.pos == self.toks.len()).then_some(v)
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn eat(&mut self, op: &str) -> bool {
        if matches!(self.peek(), Some(Tok::Op(o)) if *o == op) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn or(&mut self) -> Option<f64> {
        let mut v = self.and()?;
        while self.eat("||") {
            let r = self.and()?;
            v = f64::from(v != 0.0 || r != 0.0);
        }
        Some(v)
    }

    fn and(&mut self) -> Option<f64> {
        let mut v = self.cmp()?;
        while self.eat("&&") {
            let r = self.cmp()?;
            v = f64::from(v != 0.0 && r != 0.0);
        }
        Some(v)
    }

    fn cmp(&mut self) -> Option<f64> {
        let l = self.add()?;
        for op in ["==", "!=", ">=", "<=", ">", "<"] {
            if self.eat(op) {
                let r = self.add()?;
                let t = match op {
                    "==" => (l - r).abs() < 1e-9,
                    "!=" => (l - r).abs() >= 1e-9,
                    ">=" => l >= r,
                    "<=" => l <= r,
                    ">" => l > r,
                    _ => l < r,
                };
                return Some(f64::from(t));
            }
        }
        Some(l)
    }

    fn add(&mut self) -> Option<f64> {
        let mut v = self.mul()?;
        loop {
            if self.eat("+") {
                v += self.mul()?;
            } else if self.eat("-") {
                v -= self.mul()?;
            } else {
                return Some(v);
            }
        }
    }

    fn mul(&mut self) -> Option<f64> {
        let mut v = self.unary()?;
        loop {
            if self.eat("*") {
                v *= self.unary()?;
            } else if self.eat("/") {
                let d = self.unary()?;
                v = if d == 0.0 { 0.0 } else { v / d };
            } else {
                return Some(v);
            }
        }
    }

    fn unary(&mut self) -> Option<f64> {
        if self.eat("-") {
            return self.unary().map(|v| -v);
        }
        if self.eat("(") {
            let v = self.or()?;
            return self.eat(")").then_some(v);
        }
        match self.peek()?.clone() {
            Tok::Num(n) => {
                self.pos += 1;
                Some(n)
            }
            Tok::Var(name) => {
                self.pos += 1;
                Some(self.vars.get(&name).copied().unwrap_or(0.0))
            }
            Tok::Op(_) => None,
        }
    }
}

fn tokenize(src: &str) -> Vec<Tok> {
    const OPS: [&str; 15] = ["||", "&&", "==", "!=", ">=", "<=", ">", "<", "+", "-", "*", "/", "(", ")", "="];
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() || c == '.' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            out.push(Tok::Num(chars[start..i].iter().collect::<String>().parse().unwrap_or(0.0)));
        } else if c == '$' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            out.push(Tok::Var(chars[start..i].iter().collect()));
        } else if let Some(op) = OPS.iter().find(|op| chars[i..].iter().take(op.len()).copied().eq(op.chars())) {
            out.push(Tok::Op(op));
            i += op.len();
        } else {
            // Unknown character: make the expression unreadable rather than guess.
            out.push(Tok::Op("?"));
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(pairs: &[(&str, f64)]) -> HashMap<String, f64> {
        pairs.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect()
    }

    #[test]
    fn conditions() {
        let v = vars(&[("$a", 2.0), ("$b", 1.0)]);
        assert!(eval_condition("", &v));
        assert!(eval_condition("$a>=2&&$b==1", &v));
        assert!(!eval_condition("$a>2 || $b!=1", &v));
        assert!(eval_condition("($a+$b)*2==6", &v));
        assert!(eval_condition("$missing==0", &v));
        assert!(eval_condition("$a >= -1", &v));
        assert!(eval_condition("$a ?? 1", &v), "unreadable counts as true");
    }

    #[test]
    fn actions() {
        let mut v = vars(&[("$a", 2.0)]);
        apply_action("$a=$a+1;$b=10; $c = $a*2 ;", &mut v);
        assert_eq!((v["$a"], v["$b"], v["$c"]), (3.0, 10.0, 6.0));
        apply_action("$a==1;nonsense", &mut v);
        assert_eq!(v["$a"], 3.0);
    }
}
