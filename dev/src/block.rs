//! Generated blocks inside hand-written files, between
//! `<!-- BEGIN name ... -->` and `<!-- END name -->`.

/// `doc` with block `name` replaced by `block` (markers included); `None`
/// when `doc` has no such block.
#[must_use]
pub(crate) fn splice(doc: &str, name: &str, block: &str) -> Option<String> {
    let end = format!("<!-- END {name} -->");
    let start = doc.find(&format!("<!-- BEGIN {name}"))?;
    let len = doc.get(start..)?.find(&end)?.checked_add(end.len())?;
    let stop = start.checked_add(len)?;
    let after = doc.get(stop..)?;
    let rest = after.strip_prefix('\n').unwrap_or(after);
    Some(format!("{}{block}{rest}", doc.get(..start)?))
}

#[cfg(test)]
mod tests;
