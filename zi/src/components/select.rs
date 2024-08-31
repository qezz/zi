use std::{cmp, iter};

use super::text::{Text, TextProperties};
use crate::{
    terminal::input::ExtraKey, Bindings, Callback, Component, ComponentExt, ComponentLink,
    FlexDirection, Item, Key, Layout, Rect, ShouldRender, Style,
};

#[derive(Clone, PartialEq)]
pub struct SelectProperties {
    pub background: Style,
    pub direction: FlexDirection,
    pub focused: bool,
    pub item_at: Callback<usize, Item>,
    pub num_items: usize,
    pub item_size: usize,
    pub selected: usize,
    pub on_change: Option<Callback<usize>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    NextItem,
    PreviousItem,
    FirstItem,
    LastItem,
    NextPage,
    PreviousPage,
}

pub struct Select {
    properties: SelectProperties,
    frame: Rect,
    offset: usize,
}

impl Select {
    fn ensure_selected_item_in_view(&mut self) {
        let selected = self.properties.selected;
        let num_visible_items = self.frame.size.height / self.properties.item_size;

        // Compute offset
        self.offset = cmp::min(self.offset, selected);
        if selected - self.offset >= num_visible_items.saturating_sub(1) {
            self.offset = selected + 1 - num_visible_items;
        } else if selected < self.offset {
            self.offset = selected;
        }
    }
}

impl Component for Select {
    type Message = Message;
    type Properties = SelectProperties;

    fn create(properties: Self::Properties, frame: Rect, _link: ComponentLink<Self>) -> Self {
        let mut select = Self {
            properties,
            frame,
            offset: 0,
        };
        select.ensure_selected_item_in_view();
        select
    }

    fn change(&mut self, properties: Self::Properties) -> ShouldRender {
        if self.properties != properties {
            self.properties = properties;
            self.ensure_selected_item_in_view();
            ShouldRender::Yes
        } else {
            ShouldRender::No
        }
    }

    fn resize(&mut self, frame: Rect) -> ShouldRender {
        self.frame = frame;
        self.ensure_selected_item_in_view();
        ShouldRender::Yes
    }

    fn update(&mut self, message: Self::Message) -> ShouldRender {
        let current_selected = self.properties.selected;
        let new_selected = match (message, self.is_reversed()) {
            (Message::NextItem, false) | (Message::PreviousItem, true) => cmp::min(
                current_selected + 1,
                self.properties.num_items.saturating_sub(1),
            ),
            (Message::PreviousItem, false) | (Message::NextItem, true) => {
                current_selected.saturating_sub(1)
            }
            (Message::FirstItem, false) | (Message::LastItem, true) => 0,
            (Message::LastItem, false) | (Message::FirstItem, true) => {
                self.properties.num_items.saturating_sub(1)
            }
            (Message::NextPage, false) | (Message::PreviousPage, true) => cmp::min(
                current_selected + self.frame.size.height,
                self.properties.num_items.saturating_sub(1),
            ),
            (Message::PreviousPage, false) | (Message::NextPage, true) => {
                current_selected.saturating_sub(self.frame.size.height)
            }
        };
        if current_selected != new_selected {
            if let Some(on_change) = self.properties.on_change.as_mut() {
                on_change.emit(new_selected)
            }
        }
        ShouldRender::No
    }

    fn view(&self) -> Layout {
        let num_visible_items = cmp::min(
            self.properties.num_items.saturating_sub(self.offset),
            self.frame.size.height / self.properties.item_size,
        );
        let items = (self.offset..)
            .take(num_visible_items)
            .map(|index| self.properties.item_at.emit(index));

        if self.properties.item_size * num_visible_items < self.frame.size.height {
            // "Filler" component for the unused space
            let spacer = iter::once(Item::auto(Text::with(
                TextProperties::new().style(self.properties.background),
            )));
            Layout::container(self.properties.direction, items.chain(spacer))
        } else {
            Layout::container(self.properties.direction, items)
        }
    }

    fn bindings(&self, bindings: &mut Bindings<Self>) {
        bindings.set_focus(self.properties.focused);

        if !bindings.is_empty() {
            return;
        }
        bindings.add("next-item", [Key::Ctrl('n')], || Message::NextItem);
        bindings.add("next-item", [Key::Down], || Message::NextItem);
        bindings.add("previous-item", [Key::Ctrl('p')], || Message::PreviousItem);
        bindings.add("previous-item", [Key::Up], || Message::PreviousItem);
        bindings.add("first-item", [Key::Alt(ExtraKey::Char('<'))], || {
            Message::FirstItem
        });
        bindings.add("last-item", [Key::Alt(ExtraKey::Char('>'))], || {
            Message::LastItem
        });
        bindings.add("next-page", [Key::Ctrl('v')], || Message::NextPage);
        bindings.add("next-page", [Key::PageDown], || Message::NextPage);
        bindings.add("previous-page", [Key::Alt(ExtraKey::Char('v'))], || {
            Message::PreviousPage
        });
        bindings.add("previous-page", [Key::PageUp], || Message::PreviousPage);
    }
}

impl Select {
    fn is_reversed(&self) -> bool {
        self.properties.direction.is_reversed()
    }
}
