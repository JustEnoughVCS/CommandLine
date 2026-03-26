pub enum CompletionResult {
    FileCompletion,
    Suggestions(Vec<CompletionSuggestion>),
}

impl CompletionResult {
    /// Creates a `CompletionResult` representing file completion.
    pub fn file_comp() -> Self {
        CompletionResult::FileCompletion
    }

    /// Creates a `CompletionResult` with an empty suggestions list.
    pub fn empty_comp() -> CompletionSuggestInsert {
        CompletionSuggestInsert {
            suggestions: Vec::new(),
        }
    }

    /// Returns `true` if this is a `FileCompletion` variant.
    pub fn is_file(&self) -> bool {
        matches!(self, Self::FileCompletion)
    }

    /// Returns `true` if this is a `Suggestions` variant.
    pub fn is_suggestion(&self) -> bool {
        matches!(self, Self::Suggestions(_))
    }

    /// Returns a reference to the suggestions vector if this is a `Suggestions` variant,
    /// otherwise returns `None`.
    pub fn suggestion(&self) -> Option<&Vec<CompletionSuggestion>> {
        match self {
            Self::FileCompletion => None,
            Self::Suggestions(v) => Some(v),
        }
    }

    /// Returns the unit value if this is a `FileCompletion` variant, otherwise panics.
    pub fn unwrap_file(&self) {
        match self {
            Self::FileCompletion => (),
            Self::Suggestions(_) => {
                panic!("called `CompletionResult::unwrap_file()` on a `Suggestions` value")
            }
        }
    }

    /// Returns a reference to the suggestions vector if this is a `Suggestions` variant,
    /// otherwise panics.
    pub fn unwrap_suggestion(&self) -> &Vec<CompletionSuggestion> {
        match self {
            Self::FileCompletion => {
                panic!("called `CompletionResult::unwrap_suggestion()` on a `FileCompletion` value")
            }
            Self::Suggestions(v) => v,
        }
    }

    /// Returns the unit value if this is a `FileCompletion` variant, otherwise returns `default`.
    pub fn unwrap_file_or(&self, default: ()) {
        match self {
            Self::FileCompletion => (),
            Self::Suggestions(_) => default,
        }
    }

    /// Returns a reference to the suggestions vector if this is a `Suggestions` variant,
    /// otherwise returns `default`.
    pub fn unwrap_suggestion_or<'a>(
        &'a self,
        default: &'a Vec<CompletionSuggestion>,
    ) -> &'a Vec<CompletionSuggestion> {
        match self {
            Self::FileCompletion => default,
            Self::Suggestions(v) => v,
        }
    }

    /// Returns the unit value if this is a `FileCompletion` variant, otherwise calls `f`.
    pub fn unwrap_file_or_else<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        match self {
            Self::FileCompletion => (),
            Self::Suggestions(_) => f(),
        }
    }

    /// Returns a reference to the suggestions vector if this is a `Suggestions` variant,
    /// otherwise calls `f`.
    pub fn unwrap_suggestion_or_else<'a, F>(&'a self, f: F) -> &'a Vec<CompletionSuggestion>
    where
        F: FnOnce() -> &'a Vec<CompletionSuggestion>,
    {
        match self {
            Self::FileCompletion => f(),
            Self::Suggestions(v) => v,
        }
    }
}

pub struct CompletionSuggestInsert {
    suggestions: Vec<CompletionSuggestion>,
}

impl CompletionSuggestInsert {
    /// Adds a suggestion with the given text and no description.
    pub fn with_suggest<S: Into<String>>(mut self, suggest: S) -> Self {
        self.suggestions.push(CompletionSuggestion {
            suggest: suggest.into(),
            description: None,
        });
        self
    }

    /// Adds a suggestion with the given text and description.
    pub fn with_suggest_desc<S: Into<String>, D: Into<String>>(
        mut self,
        suggest: S,
        description: D,
    ) -> Self {
        self.suggestions.push(CompletionSuggestion {
            suggest: suggest.into(),
            description: Some(description.into()),
        });
        self
    }
}

impl From<CompletionSuggestInsert> for CompletionResult {
    fn from(insert: CompletionSuggestInsert) -> Self {
        CompletionResult::Suggestions(insert.suggestions)
    }
}

impl From<Vec<String>> for CompletionResult {
    fn from(strings: Vec<String>) -> Self {
        CompletionResult::Suggestions(
            strings
                .into_iter()
                .map(|s| CompletionSuggestion {
                    suggest: s,
                    description: None,
                })
                .collect(),
        )
    }
}

impl std::fmt::Display for CompletionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompletionResult::FileCompletion => write!(f, "_file_"),
            CompletionResult::Suggestions(suggestions) => {
                let suggestions_str: Vec<String> =
                    suggestions.iter().map(|s| s.suggest.clone()).collect();
                write!(f, "{}", suggestions_str.join(", "))
            }
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CompletionSuggestion {
    pub suggest: String,
    pub description: Option<String>,
}

impl std::ops::Deref for CompletionSuggestion {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.suggest
    }
}

impl std::ops::DerefMut for CompletionSuggestion {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.suggest
    }
}

impl AsRef<str> for CompletionSuggestion {
    fn as_ref(&self) -> &str {
        &self.suggest
    }
}

impl PartialOrd for CompletionSuggestion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CompletionSuggestion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (&self.description, &other.description) {
            (None, None) => self.suggest.cmp(&other.suggest),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), Some(_)) => self.suggest.cmp(&other.suggest),
        }
    }
}
