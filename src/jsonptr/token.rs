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

    /// This is performed by first transforming any
    /// occurrence of the sequence '~1' to '/', and then transforming any
    /// occurrence of the sequence '~0' to '~'
    ///
    /// the string '~01' correctly becomes '~1' after transformation
    /// ```
    /// let res = JsonPointer::unescape("~01");
    /// assert_eq!(res, "~1".to_string());
    /// ```
    fn unescape(s: &str) -> String {
        s.replace("~1", "/").replace("~0", "~")
    }

    fn escape(s: &str) -> String {
        s.replace('~', "~0").replace('/', "~1")
    }

    pub fn new(s: &str) -> Self {
        Token {
            val: Self::unescape(s),
        }
    }
}

impl From<&str> for Token {
    fn from(s: &str) -> Self {
        Token::new(s)
    }
}

impl From<String> for Token {
    fn from(s: String) -> Self {
        From::from(s.as_ref())
    }
}
