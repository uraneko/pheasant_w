use hashbrown::HashSet;
// module TODO

pub mod www_authenticate {
    use hashbrown::HashSet;

    pub struct WWWAuthenticate {
        challenges: HashSet<Challenge>,
    }

    pub enum Challenge {}
}
