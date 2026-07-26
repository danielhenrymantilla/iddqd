use iddqd::{Feed, ForLt, TriHashItem, TriHashMap, tri_upcast};

#[derive(Debug)]
struct Item {
    id: u32,
    key2: u32,
    key3: u32,
}

impl TriHashItem for Item {
    type K1 = ForLt![<'a> = u32];
    type K2 = ForLt![<'a> = u32];
    type K3 = ForLt![<'a> = u32];

    fn key1(&self) -> Feed<'_, Self::K1> {
        self.id
    }

    fn key2(&self) -> Feed<'_, Self::K2> {
        self.key2
    }

    fn key3(&self) -> Feed<'_, Self::K3> {
        self.key3
    }

    tri_upcast!();
}

fn main() {
    let mut map = TriHashMap::<Item>::new();
    map.insert_unique(Item { id: 0, key2: 10, key3: 20 }).unwrap();

    let mut stashed = Vec::new();
    map.retain(|item| {
        stashed.push(item);
        false
    });

    stashed[0].id = 1;
}
