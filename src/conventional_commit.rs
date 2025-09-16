use crate::bump_type::BumpType;

#[derive(Debug, Clone, PartialEq)]
pub struct ConventionalCommit {
    pub commit_type: String,
    pub scope: Option<String>,
    pub description: String,
    pub body: Option<String>,
    pub footer: Option<String>,
    pub breaking_change: bool,
}

impl ConventionalCommit {
    pub fn parse(message: &str) -> Result<Self, String> {
        let lines: Vec<&str> = message.split('\n').collect();
        let header = lines[0];

        // Parse header: type(scope)!: description
        let breaking_change = header.contains('!');

        // Find the colon
        let colon_pos = header.find(':').ok_or("Invalid format: missing ':'")?;
        let (type_part, description) = header.split_at(colon_pos);
        let description = description[1..].trim().to_string();

        // Parse type and scope
        let type_part = type_part.replace('!', ""); // Remove breaking change marker
        let (commit_type, scope) = if let Some(paren_start) = type_part.find('(') {
            let paren_end = type_part
                .find(')')
                .ok_or("Invalid format: unclosed parenthesis")?;
            let commit_type = type_part[..paren_start].to_string();
            let scope = type_part[paren_start + 1..paren_end].to_string();
            (commit_type, Some(scope))
        } else {
            (type_part.to_string(), None)
        };

        // Parse body and footer
        let mut body = None;
        let mut footer = None;

        if lines.len() > 1 {
            let mut body_lines = Vec::new();
            let mut footer_lines = Vec::new();
            let mut in_footer = false;

            for line in &lines[1..] {
                if line.trim().is_empty() {
                    continue;
                }

                // Check if this is a footer (BREAKING CHANGE: or token: value)
                if line.contains("BREAKING CHANGE:")
                    || (line.contains(':')
                        && line
                            .chars()
                            .take_while(|c| c.is_alphabetic() || *c == '-')
                            .count()
                            > 0)
                {
                    in_footer = true;
                }

                if in_footer {
                    footer_lines.push(*line);
                } else {
                    body_lines.push(*line);
                }
            }

            if !body_lines.is_empty() {
                body = Some(body_lines.join("\n"));
            }
            if !footer_lines.is_empty() {
                footer = Some(footer_lines.join("\n"));
            }
        }

        // Check for BREAKING CHANGE in footer
        let breaking_change = breaking_change
            || footer
                .as_ref()
                .map_or(false, |f| f.contains("BREAKING CHANGE:"));

        Ok(ConventionalCommit {
            commit_type,
            scope,
            description,
            body,
            footer,
            breaking_change,
        })
    }

    pub fn bump_type(&self) -> BumpType {
        if self.breaking_change {
            BumpType::Major
        } else {
            match self.commit_type.as_str() {
                "feat" => BumpType::Minor,
                "fix" | "perf" | "security" => BumpType::Patch,
                "docs" | "style" | "refactor" | "test" | "chore" | "build" | "ci" | "revert" => {
                    BumpType::None
                }
                _ => BumpType::None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_commit() {
        let commit = ConventionalCommit::parse("feat: add new feature").unwrap();
        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.scope, None);
        assert_eq!(commit.description, "add new feature");
        assert_eq!(commit.breaking_change, false);
        assert_eq!(commit.bump_type(), BumpType::Minor);
    }

    #[test]
    fn test_commit_with_scope() {
        let commit = ConventionalCommit::parse("fix(api): resolve login issue").unwrap();
        assert_eq!(commit.commit_type, "fix");
        assert_eq!(commit.scope, Some("api".to_string()));
        assert_eq!(commit.description, "resolve login issue");
        assert_eq!(commit.bump_type(), BumpType::Patch);
    }

    #[test]
    fn test_breaking_change_with_exclamation() {
        let commit = ConventionalCommit::parse("feat!: remove deprecated API").unwrap();
        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.breaking_change, true);
        assert_eq!(commit.bump_type(), BumpType::Major);
    }

    #[test]
    fn test_breaking_change_with_scope() {
        let commit = ConventionalCommit::parse("feat(api)!: remove old endpoint").unwrap();
        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.scope, Some("api".to_string()));
        assert_eq!(commit.breaking_change, true);
        assert_eq!(commit.bump_type(), BumpType::Major);
    }

    #[test]
    fn test_commit_with_body_and_footer() {
        let message = "feat(api): add user authentication

This commit adds JWT-based authentication for users.
It includes login and logout endpoints.

BREAKING CHANGE: removes basic auth support";

        let commit = ConventionalCommit::parse(message).unwrap();
        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.scope, Some("api".to_string()));
        assert!(commit.body.is_some());
        assert!(commit.footer.is_some());
        assert_eq!(commit.breaking_change, true);
        assert_eq!(commit.bump_type(), BumpType::Major);
    }

    #[test]
    fn test_invalid_format() {
        let result = ConventionalCommit::parse("invalid message format");
        assert!(result.is_err());
    }

    #[test]
    fn test_unclosed_parenthesis() {
        let result = ConventionalCommit::parse("feat(scope: missing closing paren");
        assert!(result.is_err());
    }

    #[test]
    fn test_chore_commit() {
        let commit = ConventionalCommit::parse("chore: update dependencies").unwrap();
        assert_eq!(commit.commit_type, "chore");
        assert_eq!(commit.bump_type(), BumpType::None);
    }
}
