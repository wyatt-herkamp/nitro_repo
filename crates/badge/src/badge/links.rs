#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Links {
    pub(crate) left: Option<String>,
    pub(crate) right: Option<String>,
}

impl Links {
    pub fn is_single(&self) -> bool {
        (self.left.is_some() && self.right.is_none()) || self.is_same_link()
    }

    fn is_same_link(&self) -> bool {
        self.left.is_some() && self.left == self.right
    }

    pub fn single(&self) -> Option<&str> {
        if self.is_single() {
            Some(self.left.as_ref().unwrap())
        } else {
            None
        }
    }
    pub fn any(&self) -> bool {
        self.left.is_some() || self.right.is_some()
    }

    pub fn left(&self) -> &Option<String> {
        &self.left
    }
    pub fn right(&self) -> &Option<String> {
        &self.right
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_equal_test() {
        let l = Links {
            left: Some("example".to_string()),
            right: Some("example".to_string()),
        };
        assert!(l.is_single());
    }
}
