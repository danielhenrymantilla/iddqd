use iddqd::{BiHashItem, BiHashMap, Feed, ForLt, bi_upcast};

#[derive(Debug)]
struct Item {
    id: u32,
    key2: u32,
}

impl BiHashItem for Item {
    type K1 = ForLt![<'a> = u32];
    type K2 = ForLt![<'a> = u32];

    fn key1(&self) -> Feed<'_, Self::K1> {
        self.id
    }

    fn key2(&self) -> Feed<'_, Self::K2> {
        self.key2
    }

    bi_upcast!();
}

fn main() {
    let mut map = BiHashMap::<Item>::new();
    map.insert_unique(Item { id: 0, key2: 10 }).unwrap();

    let mut stashed = Vec::new();
    map.retain(|item| {
        stashed.push(item);
        false
    });

    stashed[0].id = 1;
}
