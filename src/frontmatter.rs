use serde_yaml::Result;

/// YAML front matter from an Obsidian note.
///
/// This is essentially an alias of [`serde_yaml::Mapping`] so all the methods available on that type
/// are available with `Frontmatter` as well.
///
/// # Examples
///
/// ```
/// # use obsidian_export::Frontmatter;
/// use serde_yaml::Value;
///
/// let mut frontmatter = Frontmatter::new();
/// let key = Value::String("foo".to_string());
///
/// frontmatter.insert(
///     key.clone(),
///     Value::String("bar".to_string()),
/// );
///
/// assert_eq!(
///     frontmatter.get(&key),
///     Some(&Value::String("bar".to_string())),
/// )
/// ```
pub type Frontmatter = serde_yaml::Mapping;

// Would be nice to rename this to just from_str, but that would be a breaking change.
#[allow(clippy::module_name_repetitions)]
pub fn frontmatter_from_str(mut s: &str) -> Result<Frontmatter> {
    if s.is_empty() {
        s = "{}";
    }
    let frontmatter: Frontmatter = serde_yaml::from_str(s)?;
    Ok(frontmatter)
}

// Would be nice to rename this to just to_str, but that would be a breaking change.
#[allow(clippy::module_name_repetitions)]
pub fn frontmatter_to_str(frontmatter: &Frontmatter) -> Result<String> {
    if frontmatter.is_empty() {
        return Ok("---\n---\n".to_owned());
    }

    let mut buffer = String::new();
    buffer.push_str("---\n");
    buffer.push_str(&serde_yaml::to_string(&frontmatter)?);
    buffer.push_str("---\n");
    Ok(buffer)
}

/// Available strategies for the inclusion of frontmatter in notes.
#[derive(Debug, Clone, Copy)]
// Would be nice to rename this to just Strategy, but that would be a breaking change.
#[allow(clippy::module_name_repetitions)]
#[non_exhaustive]
pub enum FrontmatterStrategy {
    /// Copy frontmatter when a note has frontmatter defined.
    Auto,
    /// Always add frontmatter header, including empty frontmatter when none was originally
    /// specified.
    Always,
    /// Never add any frontmatter to notes.
    Never,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_yaml::Value;

    #[test]
    fn empty_string_should_yield_empty_frontmatter() {
        assert_eq!(frontmatter_from_str("").unwrap(), Frontmatter::new());
    }

    #[test]
    fn empty_frontmatter_to_str() {
        let frontmatter = Frontmatter::new();
        assert_eq!(
            frontmatter_to_str(&frontmatter).unwrap(),
            format!("---\n---\n")
        );
    }

    #[test]
    fn nonempty_frontmatter_to_str() {
        let mut frontmatter = Frontmatter::new();
        frontmatter.insert(
            Value::String("foo".to_string()),
            Value::String("bar".to_string()),
        );
        assert_eq!(
            frontmatter_to_str(&frontmatter).unwrap(),
            format!("---\nfoo: bar\n---\n")
        );
    }
}
