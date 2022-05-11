//! Configuration options related to formatting comments.
use crate::constants::DEFAULT_MAX_COMMENT_WIDTH;

use super::user_opts::CommentsOptions;

#[derive(Debug)]
pub struct Comments {
    /// Break comments to fit on the line.
    pub wrap_comments: bool,
    /// Maximum length of comments. No effect unless wrap_comments = true.
    pub comment_width: usize,
    /// Convert /* */ comments to // comments where possible
    pub normalize_comments: bool,
}

impl Default for Comments {
    fn default() -> Self {
        Self {
            wrap_comments: false,
            comment_width: DEFAULT_MAX_COMMENT_WIDTH,
            normalize_comments: false,
        }
    }
}

impl Comments {
    pub fn from_opts(opts: &CommentsOptions) -> Self {
        Self {
            wrap_comments: opts.wrap_comments.unwrap_or(false),
            comment_width: opts.comment_width.unwrap_or(DEFAULT_MAX_COMMENT_WIDTH),
            normalize_comments: opts.normalize_comments.unwrap_or(false),
        }
    }
}
