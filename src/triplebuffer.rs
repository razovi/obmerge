use std::sync::Mutex;
pub struct TripleBuffer<T>{
    data: [Mutex<Option<T>>; 3],
    last: Option<usize>,
    time: u64
}
impl<T> TripleBuffer<T>
where T: Clone{
    pub fn new() -> Self{
        TripleBuffer{
            data: [Mutex::new(None), Mutex::new(None), Mutex::new(None)],
            last: None,
            time: 0
        }
    }
    pub fn push(&mut self, x: T, time: u64){
        let mut nl = self.last.unwrap() + 1;
        if nl == 3{
            nl = 0;
        }
        let mut elem = self.data[nl].lock().unwrap();
        *elem = Some(x);
        self.last = Some(nl);
        self.time = time;
    }
    pub fn top(&self) -> Option<T> {
        if let Some(x) = self.last{
            let elem = self.data[x].lock().unwrap();
            (*elem).clone()
        }
        else{
            None
        }
    }
    pub fn empty(&self) -> bool {
        if let Some(_) = self.last{
            true
        }
        else{
            false
        }
    }
}
