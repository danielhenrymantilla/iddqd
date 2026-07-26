use iddqd::{BiHashItem, Feed, ForLt, bi_hash_map, bi_upcast};

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
    let mut map = bi_hash_map! {
        Item { id: 0, key2: 10 },
        Item { id: 1, key2: 11 },
    };

    let mut entry = match map.entry(0, 11) {
        iddqd::bi_hash_map::Entry::Occupied(entry) => entry,
        iddqd::bi_hash_map::Entry::Vacant(_) => unreachable!(),
    };

    let mut refs = entry.get_mut();
    let mut stashed = Vec::new();
    refs.for_each(|item| {
        stashed.push(item);
    });

    stashed[0].id = 2;
}
