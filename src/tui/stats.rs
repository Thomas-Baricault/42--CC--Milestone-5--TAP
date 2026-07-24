use ratatui::prelude::Widget;

use crate::tui::ToListItem;

pub struct StatsPage {
    pub armor_unequip: crate::tui::Button,
    pub weapon_unequip: crate::tui::Button,
    pub inventory: crate::tui::List<crate::game::Item>,
}

impl StatsPage {
    pub fn update(&mut self, knowledge: &crate::tui::Knowledge) {
        self.inventory.items = crate::game::Item::items_from_iter(&knowledge.player.items);
    }
}

impl Default for StatsPage {
    fn default() -> Self {
        Self {
            armor_unequip: crate::tui::Button::new("Unequip"),
            weapon_unequip: crate::tui::Button::new("Unequip"),
            inventory: crate::tui::List::new("Inventory"),
        }
    }
}

impl crate::tui::NotebookPage for StatsPage {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> &str {
        "STATS"
    }
}

impl crate::tui::Widget for StatsPage {
    fn render_with_data(&mut self, knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let [top, bottom] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(7),
            ratatui::layout::Constraint::Fill(1),
        ])
            .areas(area);
        self.inventory.render_with_data(knowledge, bottom, buf);
        let block = ratatui::widgets::Block::bordered()
            .title("Stats")
            .padding(ratatui::widgets::Padding::symmetric(2, 1));
        let inner = block.inner(top);
        block.render(top, buf);
        let [top, middle, bottom] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
        ])
            .areas(inner);
        ratatui::widgets::Paragraph::new(format!(
            "HP: {}/{}",
            knowledge.player.status.hp,
            knowledge.player.status.max_hp,
        ))
            .render(top, buf);
        let [left, right] = ratatui::layout::Layout::horizontal([
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Length(self.armor_unequip.width()),
        ])
            .areas(middle);
        ratatui::widgets::Paragraph::new(format!(
            "Armor: {}",
            if knowledge.player.status.armor.is_empty() {
                "None".to_string()
            } else {
                format!(
                    "{} ({})",
                    knowledge.item_name(&knowledge.player.status.armor.clone()),
                    if let Some(item) = knowledge.items.get(&knowledge.player.status.armor) && let crate::game::ItemKind::Armor { armor } = &item.data {
                        armor.to_string()
                    } else {
                        "?".to_string()
                    }
                )
            },
        ))
            .render(left, buf);
        self.armor_unequip.render(right, buf);
        let [left, right] = ratatui::layout::Layout::horizontal([
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Length(self.weapon_unequip.width()),
        ])
            .areas(bottom);
        ratatui::widgets::Paragraph::new(format!(
            "Weapon: {}",
            if knowledge.player.status.weapon.is_empty() {
                "None".to_string()
            } else {
                format!(
                    "{} ({})",
                    knowledge.item_name(&knowledge.player.status.weapon.clone()),
                    if let Some(item) = knowledge.items.get(&knowledge.player.status.weapon) && let crate::game::ItemKind::Weapon { damage } = &item.data {
                        damage.to_string()
                    } else {
                        "?".to_string()
                    }
                )
            },
        ))
            .render(left, buf);
        self.weapon_unequip.render(right, buf);
    }
}
