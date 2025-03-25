use cli_clipboard::{ClipboardContext, ClipboardProvider};
use sublime_fuzzy::best_match;

use crate::data::Bookmark;
use crate::errors::{Error, Result};

pub fn copy(text: String) -> Result<()> {
    let mut ctx = ClipboardContext::new().map_err(|e| Error::ClipboardNotFound(e.to_string()))?;
    println!("{}", text);
    ctx.set_contents(text)
        .map_err(|e| Error::ClipboardCopyError(e.to_string()))?;
    Ok(())
}

pub fn fuzz<'a>(query: &str, store: &Vec<Bookmark>) -> usize {
    let (id, _) = store
        .iter()
        .filter_map(|i| best_match(query, &i.title))
        .enumerate()
        .max_by(|(_, a), (_, b)| a.score().cmp(&b.score()))
        .unwrap();
    id
}
