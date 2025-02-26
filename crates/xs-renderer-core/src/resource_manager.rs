use std::{collections::HashMap, ops::{Index, IndexMut}};


#[derive(Clone)]
pub struct ResourceManager<T> {
    id: usize,
    resources: HashMap<usize, T>,
}

impl<T> Default for ResourceManager<T> {
    fn default() -> Self {
        Self {
            id: 0,
            resources: HashMap::new(),
        }
    }
}

impl<T> ResourceManager<T> {
    pub fn add(&mut self, resource: T) -> usize {
        self.resources.insert(self.id, resource);
        self.id += 1;
        self.id - 1
    }

    pub fn remove(&mut self, id: usize) -> T {
        // Let it crash if the resource is not found
        self.resources.remove(&id).unwrap()
    }
}

impl<T> Index<usize> for ResourceManager<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.resources[&index]
    }
}

impl<T> IndexMut<usize> for ResourceManager<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.resources.get_mut(&index).unwrap()
    }
}
