use std::collections::HashSet;

pub enum MonoPoly<T> {
    Mono(T),
    Poly(Vec<T>),
    Unique(HashSet<T>),
    None,
}

impl<T> MonoPoly<T>
where
    T: Eq + std::hash::Hash,
{
    pub fn update(&mut self, t: T) -> bool {
        let Self::Mono(old) = self else { return false };
        *old = t;

        true
    }

    pub fn insert(&mut self, t: T) -> bool {
        let Self::Unique(set) = self else {
            return false;
        };
        set.insert(t);

        true
    }

    pub fn push(&mut self, t: T) -> bool {
        let Self::Poly(vec) = self else { return false };
        vec.push(t);

        true
    }

    pub fn extend<I>(&mut self, i: I) -> bool
    where
        I: Iterator<Item = T>,
    {
        match self {
            Self::Poly(vec) => vec.extend(i),
            Self::Unique(set) => set.extend(i),
            Self::Mono(_) | Self::None => return false,
        }

        true
    }

    pub fn is_none(&self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(&Self::None)
    }
}
