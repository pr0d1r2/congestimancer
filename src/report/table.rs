//! [`write_status`]: the `status` table for humans (src/report:T2, V5,
//! V6).

use std::io::{self, Write};

use super::{Paint, StatusReport, StatusRow, Style};
use crate::signal::Reading;

/// The gap between columns.
const GAP: &str = "  ";

/// Writes `report`'s signals as an aligned table, one row per signal,
/// unsupported ones with their reason (src/report:V5).
///
/// # Errors
///
/// When `out` fails to take the bytes.
pub fn write_status<W: Write + ?Sized>(
    out: &mut W,
    report: &StatusReport,
    style: Style,
) -> io::Result<()> {
    writeln!(out, "{} {}", style.paint("host", Paint::Bold), report.host)?;
    if !report.signals.is_empty() {
        let rows: Vec<Cells> = report.signals.iter().map(Cells::of).collect();
        let header = Cells::header(rows.iter().any(|r| !r.trend.is_empty()));
        let widths = Widths::of(&header, &rows);
        let head = widths.line(&header);
        writeln!(out, "{}", style.paint(&head, Paint::Bold))?;
        for row in &rows {
            writeln!(out, "{}", widths.painted(row, style))?;
        }
    }
    write_learned(out, report, style)
}

/// The learned memory peaks, one per line under a heading; nothing when
/// there are none.
fn write_learned<W: Write + ?Sized>(
    out: &mut W,
    report: &StatusReport,
    style: Style,
) -> io::Result<()> {
    if report.learned.is_empty() {
        return Ok(());
    }
    let heading = style.paint("learned memory peaks", Paint::Bold);
    writeln!(out, "\n{heading}")?;
    let width = report.learned.iter().map(|p| p.key.len()).max();
    for peak in &report.learned {
        let pad = width.unwrap_or(0);
        writeln!(out, "{:<pad$}  {}", peak.key, peak.bytes)?;
    }
    Ok(())
}

/// One table row as text.
struct Cells {
    /// The signal's switch name.
    id: String,
    /// Where it was read.
    source: String,
    /// The value, `warming up`, or `unsupported: <reason>`.
    value: String,
    /// The trend, or empty.
    trend: String,
    /// How the value is drawn, if it stands out.
    paint: Option<Paint>,
    /// Whether the value is a reason, which may run past its column.
    overflows: bool,
}

impl Cells {
    /// The row for `row`.
    fn of(row: &StatusRow) -> Self {
        let (paint, overflows) = match row.reading {
            Reading::Value(_) => (None, false),
            Reading::Warming => (Some(Paint::Dim), false),
            Reading::Unsupported { .. } => (Some(Paint::Yellow), true),
        };
        Self {
            id: row.id.name().to_owned(),
            source: row.source.clone().into_owned(),
            value: row.reading.to_string(),
            trend: row.trend.map(|t| t.to_string()).unwrap_or_default(),
            paint,
            overflows,
        }
    }

    /// The header row; `TREND` only when a row has one.
    fn header(trends: bool) -> Self {
        Self {
            id: "SIGNAL".to_owned(),
            source: "SOURCE".to_owned(),
            value: "VALUE".to_owned(),
            trend: if trends { "TREND" } else { "" }.to_owned(),
            paint: None,
            overflows: false,
        }
    }
}

/// Column widths.
struct Widths {
    /// The signal column.
    id: usize,
    /// The source column.
    source: usize,
    /// The value column, unsupported reasons left out.
    value: usize,
}

impl Widths {
    /// The widths that fit `header` and every row of `rows`.
    fn of(header: &Cells, rows: &[Cells]) -> Self {
        let all = || std::iter::once(header).chain(rows);
        let widest = |cell: fn(&Cells) -> &str| {
            all().map(|c| cell(c).chars().count()).max().unwrap_or(0)
        };
        Self {
            id: widest(|c| &c.id),
            source: widest(|c| &c.source),
            value: all()
                .filter(|c| !c.overflows)
                .map(|c| c.value.chars().count())
                .max()
                .unwrap_or(0),
        }
    }

    /// `cells` laid out, value as given.
    fn line(&self, cells: &Cells) -> String {
        self.layout(cells, &cells.value)
    }

    /// `cells` laid out, value drawn in `style`.
    fn painted(&self, cells: &Cells, style: Style) -> String {
        let value = cells.paint.map_or_else(
            || cells.value.clone(),
            |paint| style.paint(&cells.value, paint),
        );
        self.layout(cells, &value)
    }

    /// The row with `value` in the value column; the value is padded only
    /// when a trend follows it, so no line ends in blanks.
    fn layout(&self, cells: &Cells, value: &str) -> String {
        let Cells { id, source, .. } = cells;
        let mut line = format!(
            "{id:<iw$}{GAP}{source:<sw$}{GAP}{value}",
            iw = self.id,
            sw = self.source,
        );
        if !cells.trend.is_empty() {
            let pad = self.value.saturating_sub(cells.value.chars().count());
            line.push_str(&" ".repeat(pad));
            line.push_str(GAP);
            line.push_str(&cells.trend);
        }
        line
    }
}

#[cfg(test)]
mod tests;
