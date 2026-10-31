//! Guilds the user follows, treated as favourites (RF-73).
//!
//! Identity is the guild id, never the name: names change and several guilds share tags.
//! The list is serializable so the config layer only has to store it.

use serde::{Deserialize, Serialize};

use crate::matchup::GuildSummary;

/// How many guilds can be followed at once.
///
/// Every followed guild is a ring on the map, a filter on the alerts and a row in the settings
/// list; the list used to be every guild of the matchup, which is what made it long and slow
/// (report of 2026-10-31). Ten is more than anyone watches.
pub const MAX_FOLLOWED: usize = 10;

/// Ordered, duplicate-free list of followed guild ids.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FollowList {
    ids: Vec<String>,
}

impl FollowList {
    /// Empty list.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the guild is followed.
    pub fn contains(&self, guild: &str) -> bool {
        self.ids.iter().any(|g| g == guild)
    }

    /// Follows a guild. Returns `false` if it already was followed or the list is full.
    pub fn add(&mut self, guild: &str) -> bool {
        if self.contains(guild) || self.is_full() {
            return false;
        }
        self.ids.push(guild.to_string());
        true
    }

    /// Whether the list already has [`MAX_FOLLOWED`] guilds.
    pub fn is_full(&self) -> bool {
        self.ids.len() >= MAX_FOLLOWED
    }

    /// Stops following a guild. Returns `false` if it was not followed.
    pub fn remove(&mut self, guild: &str) -> bool {
        match self.ids.iter().position(|g| g == guild) {
            Some(at) => {
                self.ids.remove(at);
                true
            }
            None => false,
        }
    }

    /// Adds or removes, returning the new state.
    ///
    /// With the list full it does nothing and keeps saying `false`: the UI has to explain it.
    pub fn toggle(&mut self, guild: &str) -> bool {
        if self.remove(guild) {
            false
        } else {
            self.add(guild)
        }
    }

    /// Followed guilds, in the order they were added.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.ids.iter().map(String::as_str)
    }

    /// Number of followed guilds.
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Whether nothing is followed.
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

/// Puts followed guilds first, keeping the recency order inside each group (RF-73).
pub fn favourites_first(ranked: &[GuildSummary], follow: &FollowList) -> Vec<GuildSummary> {
    let mut out: Vec<GuildSummary> = Vec::with_capacity(ranked.len());
    out.extend(ranked.iter().filter(|g| follow.contains(&g.guild)).cloned());
    out.extend(
        ranked
            .iter()
            .filter(|g| !follow.contains(&g.guild))
            .cloned(),
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MapRole;
    use chrono::DateTime;

    fn summary(guild: &str, claims: usize, minute: u32) -> GuildSummary {
        GuildSummary {
            guild: guild.to_string(),
            claims,
            last_at: Some(
                DateTime::from_timestamp(1_800_000_000 + i64::from(minute) * 60, 0).unwrap(),
            ),
            last_map: Some((38, MapRole::Center)),
        }
    }

    #[test]
    fn rf73_the_list_stops_at_ten_guilds() {
        let mut list = FollowList::new();
        for index in 0..MAX_FOLLOWED {
            assert!(list.add(&format!("g{index}")), "caben diez");
        }
        assert!(list.is_full());
        assert!(
            !list.add("una-mas"),
            "el onceavo no entra: la lista no puede crecer sin fin"
        );
        assert!(!list.toggle("otra-mas"), "toggle tampoco lo cuela");
        assert_eq!(list.len(), MAX_FOLLOWED);

        assert!(!list.toggle("g0"), "quitar sigue funcionando");
        assert!(list.add("una-mas"), "con un hueco vuelve a entrar");
    }

    #[test]
    fn rf73_add_is_idempotent_and_toggle_flips() {
        let mut list = FollowList::new();
        assert!(list.add("A"));
        assert!(!list.add("A"), "añadir dos veces no duplica");
        assert_eq!(list.len(), 1);

        assert!(!list.toggle("A"), "toggle quita si estaba");
        assert!(list.is_empty());
        assert!(list.toggle("A"), "toggle añade si no estaba");
        assert!(list.contains("A"));
    }

    #[test]
    fn rf73_remove_reports_whether_it_did_anything() {
        let mut list = FollowList::new();
        list.add("A");
        assert!(list.remove("A"));
        assert!(!list.remove("A"));
    }

    #[test]
    fn rf73_list_survives_serde() {
        let mut list = FollowList::new();
        list.add("1DB147F0");
        list.add("8AE90FC1");

        let json = serde_json::to_string(&list).unwrap();
        assert_eq!(json, r#"["1DB147F0","8AE90FC1"]"#);

        let back: FollowList = serde_json::from_str(&json).unwrap();
        assert_eq!(back, list);
    }

    #[test]
    fn rf73_favourites_come_first_keeping_recency_inside_each_group() {
        let ranked = vec![
            summary("A", 1, 30),
            summary("B", 2, 20),
            summary("C", 1, 10),
        ];
        let mut follow = FollowList::new();
        follow.add("C");

        let ordered = favourites_first(&ranked, &follow);
        let ids: Vec<&str> = ordered.iter().map(|g| g.guild.as_str()).collect();
        assert_eq!(ids, ["C", "A", "B"]);
    }

    #[test]
    fn rf73_following_does_not_resurrect_unclaimed_guilds() {
        // La lista de favoritos no inventa filas: si la guild no tiene claims ahora,
        // no aparece en la lista de RF-71.
        let ranked = vec![summary("A", 1, 30)];
        let mut follow = FollowList::new();
        follow.add("Z");

        let ordered = favourites_first(&ranked, &follow);
        assert_eq!(ordered.len(), 1);
        assert_eq!(ordered[0].guild, "A");
    }
}
