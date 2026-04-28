//! Shared SQL fragment for canonical `` DEFINER=`<user>`@`<host>` `` reconstruction.
//!
//! The `information_schema` `DEFINER` column stores the value as the unquoted
//! string `user@host`; the user portion may itself contain a literal `@` (e.g.
//! `'foo@bar'@'localhost'` → `foo@bar@localhost`). Splitting on the **last**
//! `@` therefore correctly separates user from host. Both components are
//! backtick-quoted with embedded backticks doubled, matching the canonical
//! `SHOW CREATE TRIGGER` / `SHOW CREATE FUNCTION` output form.
//!
//! Used by `list_tables.rs` (`triggers_info` CTE), `list_triggers.rs`
//! (detailed-mode `definition` projection), and `list_functions.rs`
//! (detailed-mode `definition` projection) so the canonical form lives in
//! exactly one place.

/// Returns the comma-separated SQL chunks for the canonical `DEFINER` form.
///
/// The result is five comma-separated SQL expressions intended to be embedded
/// inside a `CONCAT(...)` projection: the literal `'CREATE DEFINER=...'` opener,
/// the user component (everything up to the last `@`, backtick-escaped), the
/// `...@...` separator, the host component (everything after the last `@`,
/// backtick-escaped), and the closing backtick literal.
///
/// # Examples
///
/// ```ignore
/// let sql = definer_canonical_sql("tr.DEFINER");
/// assert!(sql.contains("tr.DEFINER"));
/// ```
#[must_use]
pub(crate) fn definer_canonical_sql(col: &str) -> String {
    format!(
        "'CREATE DEFINER=`', \
         REPLACE(LEFT({col}, LENGTH({col}) - LENGTH(SUBSTRING_INDEX({col}, '@', -1)) - 1), '`', '``'), \
         '`@`', \
         REPLACE(SUBSTRING_INDEX({col}, '@', -1), '`', '``'), \
         '`'"
    )
}

#[cfg(test)]
mod tests {
    use super::definer_canonical_sql;

    #[test]
    fn renders_for_tr_definer_column() {
        let sql = definer_canonical_sql("tr.DEFINER");
        assert!(sql.starts_with("'CREATE DEFINER=`',"));
        assert!(sql.ends_with("'`'"));
        assert!(sql.contains("tr.DEFINER"));
    }

    #[test]
    fn renders_for_r_definer_column() {
        let sql = definer_canonical_sql("r.DEFINER");
        assert!(sql.contains("r.DEFINER"));
        assert!(!sql.contains("tr.DEFINER"));
    }

    #[test]
    fn renders_for_unqualified_definer_column() {
        let sql = definer_canonical_sql("DEFINER");
        assert!(sql.contains("DEFINER"));
        assert!(!sql.contains("tr.DEFINER"));
        assert!(!sql.contains("r.DEFINER"));
    }

    #[test]
    fn fragment_has_canonical_structure() {
        let sql = definer_canonical_sql("DEFINER");
        // Exactly one opener, one separator, one closer.
        assert_eq!(sql.matches("'CREATE DEFINER=`'").count(), 1);
        assert_eq!(sql.matches("'`@`'").count(), 1);
        // The closing `'`'` literal appears once at the end.
        assert!(
            sql.ends_with(", '`'"),
            "fragment must end with the closing backtick literal"
        );
    }
}
