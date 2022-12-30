pub struct Token {
    val: String,
}

pub enum TokenIndex {
    Index(usize),
    IndexAfterLastElem,
}

impl Token {
    pub fn as_key(&self) -> &str {
        &self.val
    }

    // TODO: maybe use Result for this, and use self defined error
    pub fn as_index(&self) -> Option<TokenIndex> {
        if self.val == "-" {
            return Some(TokenIndex::IndexAfterLastElem);
        }
        if self.val.len() != 1 {
            if self.val.trim_start_matches('0').len() != self.val.len() {
                // Leading zero
                return None;
            }
        }
        if let Ok(index) = self.val.parse::<usize>() {
            Some(TokenIndex::Index(index))
        } else {
            None
        }
    }
}
