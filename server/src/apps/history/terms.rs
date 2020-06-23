use std::ops::Index;
use std::str::FromStr;
use std::collections::{HashSet};

pub struct Terms {
    terms: Vec<Term>,
}

impl Terms {
    pub fn iter(&self) -> impl Iterator<Item=(TermId, &Term)>+DoubleEndedIterator {
        self.terms.iter().enumerate()
            .map(|(id, term)| (TermId { inner: id }, term))
    }

    pub fn len(&self) -> usize {
        self.terms.len()
    }
}

impl Index<TermId> for Terms {
    type Output = Term;

    fn index(&self, i: TermId) -> &Term {
        // safety: TermId's are guaranteed to be less than the length of our terms vector
        unsafe { self.terms.get_unchecked(i.inner) }
    }
}

impl FromStr for Terms {
    type Err = ();

    fn from_str(string: &str) -> Result<Terms, ()> {
        let terms = string.lines()
            .map(Term::from_str)
            .collect::<Result<Vec<Term>, ()>>()?;

        Ok(Terms { terms })
    }
}



#[derive(Clone, Debug)]
pub struct Term { // could also be called Pair (question, answer) or
    chapter: u8,
    section: u8,
    year_start: u16,
    year_end: u16,
    social: bool,
    political: bool,
    economic: bool,
    tag: String,
    term: String,
    definition: String,
}

impl Term {
    pub fn location(&self) -> (u8, u8) {
        (self.chapter, self.section)
    }

    pub fn obvious(&self) -> usize {
        let term_words_set: HashSet<&str> = self.term.split_whitespace().collect();
        let definition_word_set: HashSet<&str> = self.definition.split_whitespace().collect();

        term_words_set.intersection(&definition_word_set).count()
    }

    pub fn get_tag(&self) -> &str {
        &self.tag
    }

    pub fn get_term(&self) -> &str {
        &self.term
    }

    pub fn get_definition(&self) -> &str {
        &self.definition
    }
}

impl FromStr for Term {
    type Err = ();

    fn from_str(string: &str) -> Result<Term, ()> {
        fn option(string: &str) -> Option<Term> {
            let mut split = string.trim_end().split("\t");

            Some(Term {
                chapter: split.next()?.parse().ok()?,
                section: split.next()?.parse().ok()?,
                year_start: split.next()?.parse().ok()?,
                year_end: split.next()?.parse().ok()?,
                social: split.next()?.parse().ok()?,
                political: split.next()?.parse().ok()?,
                economic: split.next()?.parse().ok()?,
                tag: split.next()?.trim().to_string(),
                term: split.next()?.trim().to_string(),
                definition: split.next()?.trim().to_string(),
            })
        }

        option(string).ok_or(())
    }

}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct TermId {
    inner: usize
}

impl TermId {
    pub fn from_inner(inner: usize, terms: &Terms) -> Option<TermId> {
        if inner < terms.len() {
            Some(TermId { inner })
        } else {
            None
        }
    }

    pub fn inner(self) -> usize {
        self.inner
    }
}
