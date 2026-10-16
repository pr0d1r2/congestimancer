//! Generated blocks inside hand-written files, between
//! `<!-- BEGIN name ... -->` and `<!-- END name -->`.

/// `doc` with block `name` replaced by `block` (markers included); `None`
/// when `doc` has no such block.
#[must_use]
pub(crate) fn splice(doc: &str, name: &str, block: &str) -> Option<String> {
    let _ = (doc, name, block);
    None
}

#[cfg(test)]
mod tests;
