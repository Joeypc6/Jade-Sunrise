//! Rebinding Jade's own key mappings from its config screen, as Controls does: `set_mod_key` / `reset_mod_key`
//! (they also save `sunrise-mods.txt`). The screen's handlers read the host only, so a change is queued there and
//! made at the next client tick, where the host can be written.

use mod_api::*;
use std::sync::Mutex;

enum Op {
    Set(String, String, u32),
    Reset(String),
}

static QUEUE: Mutex<Vec<Op>> = Mutex::new(Vec::new());

fn push(op: Op) {
    if let Ok(mut q) = QUEUE.lock() {
        q.push(op);
    }
}

/// Binds Jade's key mapping `name` (its registration id) to `key` with GLFW `modifiers` (shift 1, control 2, alt 4).
pub(crate) fn set_key(name: &str, key: &str, modifiers: u32) {
    push(Op::Set(name.to_owned(), key.to_owned(), modifiers));
}

/// Puts Jade's key mapping `name` back to its default.
pub(crate) fn reset_key(name: &str) {
    push(Op::Reset(name.to_owned()));
}

/// The queued changes, in order.
pub(crate) fn tick(host: &mut dyn ClientHost) {
    let ops = match QUEUE.lock() {
        Ok(mut q) if !q.is_empty() => std::mem::take(&mut *q),
        _ => return,
    };
    for op in ops {
        let (name, result) = match op {
            Op::Set(name, key, modifiers) => {
                let r = host.set_mod_key(crate::opts::ID, &name, &key, ClientModifiers::new(modifiers));
                (name, r)
            }
            Op::Reset(name) => {
                let r = host.reset_mod_key(crate::opts::ID, &name);
                (name, r)
            }
        };
        if let Err(refusal) = result {
            eprintln!("[jade] key {name}: {refusal}");
        }
    }
}
