//! The calculator reference for content writers (docs/CALCULATORS.md).

use super::*;

/// `docs/CALCULATORS.md`: every calculator's id and inputs, for content
/// writers. A test keeps the file equal to this.
pub fn reference_markdown() -> String {
    let mut out = String::from(
        "# Calculators\n\nGenerated from `crates/core/src/calc` (run \
         `cargo test -p batna-core -- --ignored print_reference`). A `calc` item names one of these in \
         `calculator:` and gives its `inputs` by key (a key may drop its unit suffix: `alt` for \
         `alt_u_l`). `just validate` computes every item, so a typo, a \
         missing input or an `expect` that disagrees fails the check. `expect` may be the band id \
         (`child_pugh.b`) or text that contains the result (`Child-Pugh B (8 points)`, `Stage C`).\n",
    );
    for s in all() {
        out.push_str(&format!(
            "\n## `{}`: {}\n\nSource: {}\n\n| input | kind | |\n|---|---|---|\n",
            s.id, s.name, s.source
        ));
        for f in s.fields {
            let kind = match f.kind {
                FieldKind::Number { unit: "", .. } => "number".to_owned(),
                FieldKind::Number { unit, .. } => format!("number, {unit}"),
                FieldKind::Flag => "true / false (default false)".to_owned(),
                FieldKind::Choice(opts) => {
                    opts.iter().map(|(id, _)| format!("`{id}`")).collect::<Vec<_>>().join(", ")
                }
            };
            let note = if f.optional && !matches!(f.kind, FieldKind::Flag) { "optional" } else { "" };
            out.push_str(&format!(
                "| `{}` | {kind} | {}{}{} |\n",
                f.key,
                f.label,
                if note.is_empty() { "" } else { "; " },
                note
            ));
        }
    }
    out
}
