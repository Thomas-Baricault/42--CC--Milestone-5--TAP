#[derive(
    Default,
    Clone,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum ItemKind {
    #[default]
    Valuable,

    Armor {
        armor: u32,
    },
    Consumable {
        heal: u32,
    },
    Weapon {
        damage: u32,
    },
}

#[derive(
    Clone,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub description: String,

    #[serde(default)]
    pub data: ItemKind,
}
