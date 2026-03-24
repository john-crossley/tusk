use crate::models::item::Item;

pub trait HasItems {
    fn items(&self) -> &[Item];
}

pub trait TaskStats: HasItems {
    fn completed(&self) -> usize {
        self.items().iter().filter(|i| i.done_at.is_some()).count()
    }

    fn total(&self) -> usize {
        self.items().len()
    }

    fn open(&self) -> usize {
        self.total() - self.completed()
    }
}