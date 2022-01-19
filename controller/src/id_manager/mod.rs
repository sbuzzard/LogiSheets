use controller_base::{AuthorId, ExtBookId, FuncId, SheetId, TextId};
use im::hashmap::HashMap;
use num::{Num, NumCast};
use std::ops::AddAssign;

pub type SheetIdManager = IdManager<SheetId>;
pub type BookIdManager = IdManager<ExtBookId>;
pub type TextIdManager = IdManager<TextId>;
pub type FuncIdManager = IdManager<FuncId>;
pub type AuthorIdManager = IdManager<AuthorId>;
pub type NameIdManager = name_id_manager::NameIdManager;
mod name_id_manager;

#[derive(Debug, Clone)]
pub struct IdManager<T>
where
    T: Copy + Num + AddAssign + NumCast + Eq,
{
    pub next_available: T,
    pub ids: HashMap<String, T>,
}

impl<T> IdManager<T>
where
    T: Copy + Num + AddAssign + NumCast + Eq,
{
    pub fn new(start: T) -> Self {
        IdManager {
            next_available: start,
            ids: HashMap::new(),
        }
    }

    pub fn has(&self, name: &str) -> Option<T> {
        match self.ids.get(name) {
            Some(id) => Some(id.clone()),
            None => None,
        }
    }

    pub fn registry(&mut self, name: String) -> T {
        let r = self.next_available;
        self.ids.insert(name, self.next_available);
        let _1: T = NumCast::from(1usize).unwrap();
        self.next_available += _1;
        r
    }

    pub fn rename(&mut self, old_name: &str, new_name: String) {
        let result = self.ids.get(old_name);
        if result.is_none() {
            return;
        }
        let id = result.unwrap().clone();
        self.ids.remove(old_name);
        self.ids.insert(new_name, id);
    }

    pub fn get_id(&mut self, name: &str) -> T {
        match self.ids.get(name) {
            Some(r) => r.clone(),
            None => self.registry(name.to_owned()),
        }
    }

    pub fn get_string(&self, id: &T) -> Option<String> {
        let result = self.ids.iter().find(|&(_, v)| v == id);
        match result {
            Some(r) => Some(r.0.clone()),
            None => None,
        }
    }
}

impl FuncIdManager {
    pub fn get_func_id(&mut self, name: &str) -> FuncId {
        let s = name.to_uppercase();
        self.get_id(&s)
    }
}
