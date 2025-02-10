//mod vec;

/*
#[derive(Debug)]
pub struct VecDequeItem<T> {
    pub value: T,
    pub previous: Option<RefCell<Rc<VecDequeItem<T>>>>,
    pub next: Option<RefCell<Rc<VecDequeItem<T>>>>
}

pub struct VecDeque<T> {
    len: usize,
    pub front: Option<RefCell<Rc<VecDequeItem<T>>>>,
        pub back: Option<RefCell<Rc<VecDequeItem<T>>>>
}

impl<T: std::fmt::Debug> VecDeque<T> {
    //type Item;

    pub fn new() -> Self {
        VecDeque {
            len: 0,
            front: None,
            back: None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push_front(&mut self, value: T) {
        let item =
            RefCell::new(
                Rc::new(
                    VecDequeItem {
                        value,
                        previous: None,
                        next: self.front.clone()
                    }
                )
            );
        if let Some(front) = &self.front {
            front.borrow_mut().previous = Some(item);
        }
        self.front = Some(item);
    }

    pub fn push_back(&mut self, value: T) {
        let item =
            RefCell::new(
                Rc::new(
                    VecDequeItem {
                        value,
                        previous: self.back.clone(),
                        next: None
                    }
                )
            );
        if let Some(back) = &self.back {
            back.borrow_mut().next = Some(item);
        }
        self.back = Some(item);
    }

    pub fn pop_front(&mut self) -> Option<&T> {
        println!("front: {:?}", self.front);
        if let Some(front) = &self.front {
        }
        None
    }
}
*/

#[unsafe(no_mangle)]
pub extern  "C" fn main() {
    let _: Vec<u8> = Vec::new();
}
